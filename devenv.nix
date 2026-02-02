{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  dotenv.enable = true;

  # ============================================
  # PACKAGES
  # ============================================
  packages = with pkgs; [
    # git
    git

    # Rust tooling
    cargo-watch # Watch for changes and rebuild
    cargo-tarpaulin # Code coverage (Linux only, but available in nix)
    cargo-audit # Security vulnerability scanning
    cargo-nextest # Better test runner

    # Conventional commits
    cocogitto # Enforce conventional commits + changelog

    #database
    sqlx-cli

    # Fast linker
    llvmPackages.lld
    clang
  ];

  # ============================================
  # RUST TOOLCHAIN
  # ============================================
  languages.rust = {
    enable = true;
  };

  # ============================================
  # POSTGRES
  # ============================================
  services.postgres = {
    enable = true;
    listen_addresses = "127.0.0.1";
    port = 5432;
    initialDatabases = [ { name = "newsletter"; } ];
    initialScript = ''
      CREATE ROLE postgres WITH LOGIN SUPERUSER PASSWORD 'password';
    '';
    settings.max_connections = 1000;
  };

  # ============================================
  # ENVIRONMENT VARIABLES
  # ============================================
  env = {
    LANG = "en_US.UTF-8";

    # Configure cargo to use lld linker from devenv
    # This is more reliable than .cargo/config.toml for nix environments
    CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = "clang";
    CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
    CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER = "clang";
    CARGO_TARGET_X86_64_APPLE_DARWIN_RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "clang";
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=-fuse-ld=lld";

    DATABASE_URL = "postgres://postgres:password@127.0.0.1:5432/newsletter";
  };

  # ============================================
  # GIT HOOKS
  # ============================================
  git-hooks.hooks = {
    rustfmt.enable = true;

    clippy = {
      enable = true;
      settings.denyWarnings = true;
    };

    # Conventional commit validation
    cocogitto = {
      enable = true;
      name = "Conventional Commit (cocogitto)";
      entry = "bash -c 'if grep -qE \"^(fixup!|squash!|amend!)\" \"$1\"; then exit 0; fi; cog verify --file \"$1\"' --";
      stages = [ "commit-msg" ];
      pass_filenames = true;
    };
  };

  # ============================================
  # SCRIPTS
  # ============================================

  scripts.menu.exec = ''
    echo "🦀 Zero to Production - Dev Environment"
    echo "========================================"
    echo ""
    echo "Available commands:"
    echo ""
    echo "  Development:"
    echo "    dev:watch     - Watch and run check → test → run"
    echo "    dev:test      - Run tests with nextest"
    echo "    dev:coverage  - Generate coverage report"
    echo "    dev:lint      - Run clippy with -D warnings"
    echo "    dev:fmt       - Format code"
    echo "    dev:audit     - Security vulnerability scan"
    echo ""
    echo "  Commits & Releases:"
    echo "    commit        - Create conventional commit"
    echo "    cog:check     - Verify commit history"
    echo "    changelog     - Preview unreleased changes"
    echo "    release       - Create release (auto version)"
    echo "    release:patch - Force patch release"
    echo "    release:minor - Force minor release"
    echo "    release:major - Force major release"
    echo ""
  '';

  # --- Development Commands ---
  scripts."dev:watch".exec = ''
    cargo watch -x check -x "test -- --nocapture" -x run
  '';

  scripts."dev:test".exec = ''
    cargo nextest run
  '';

  scripts."dev:coverage".exec = ''
    cargo tarpaulin --ignore-tests --out Html --out Lcov
    echo "📊 Coverage report: tarpaulin-report.html"
  '';

  scripts."dev:lint".exec = ''
    cargo clippy --all-targets --all-features -- -D warnings
  '';

  scripts."dev:fmt".exec = ''
    cargo fmt
  '';

  scripts."dev:audit".exec = ''
    cargo audit
  '';

  # --- Commit Commands ---
  scripts.commit.exec = ''
    if [ $# -lt 2 ]; then
      echo "Usage: commit <type> \"<message>\" [scope]"
      echo ""
      echo "Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert"
      echo ""
      echo "Examples:"
      echo "  commit feat \"add user authentication\""
      echo "  commit fix \"handle null response\" api"
      echo "  commit docs \"update README\""
      exit 1
    fi

    TYPE="$1"
    MESSAGE="$2"
    SCOPE="''${3:-}"

    if [ -n "$SCOPE" ]; then
      FULL_MESSAGE="$TYPE($SCOPE): $MESSAGE"
    else
      FULL_MESSAGE="$TYPE: $MESSAGE"
    fi

    # Validate first
    if ! cog verify "$FULL_MESSAGE"; then
      echo "❌ Invalid commit message format"
      exit 1
    fi

    # Stage all changes if nothing staged
    if git diff --cached --quiet; then
      echo "No staged changes, staging all..."
      git add -A
    fi

    git commit -m "$FULL_MESSAGE"
  '';

  scripts."cog:check".exec = ''
    echo "🔍 Checking commit history..."
    cog check
  '';

  scripts.changelog.exec = ''
    echo "📋 Unreleased changes:"
    echo "======================"
    cog changelog
  '';

  # --- Release Commands ---
  scripts.release.exec = ''
    echo "🚀 Creating release..."

    # Check we're on main
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$BRANCH" != "main" ]] && [[ "$BRANCH" != "master" ]]; then
      echo "⚠️  Warning: Not on main branch (currently on $BRANCH)"
      read -p "Continue anyway? (y/n) " -n 1 -r
      echo
      if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
      fi
    fi

    # Check for uncommitted changes
    if ! git diff --quiet || ! git diff --cached --quiet; then
      echo "❌ You have uncommitted changes. Commit or stash them first."
      exit 1
    fi

    # Show what will be released
    echo ""
    echo "📋 Changes to be released:"
    cog changelog
    echo ""

    # Detect version
    echo "🔍 Detecting version from commits..."
    cog bump --dry-run --auto 2>&1 || true
    echo ""

    read -p "Create release? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
      exit 1
    fi

    # Create release
    cog bump --auto

    echo "✅ Released $(git describe --tags --abbrev=0)"
  '';

  scripts."release:patch".exec = ''
    echo "🔧 Creating patch release..."
    cog bump --patch
    echo "✅ Released $(git describe --tags --abbrev=0)"
  '';

  scripts."release:minor".exec = ''
    echo "✨ Creating minor release..."
    cog bump --minor
    echo "✅ Released $(git describe --tags --abbrev=0)"
  '';

  scripts."release:major".exec = ''
    echo "💥 Creating major release..."
    cog bump --major
    echo "✅ Released $(git describe --tags --abbrev=0)"
  '';

  scripts."db:migrate".exec = "sqlx migrate run";
  scripts."db:reset".exec = ''
    sqlx database drop -y || true
    sqlx database create
    sqlx migrate run
  '';
  # ============================================
  # SHELL HOOKS
  # ============================================
  enterShell = ''
    echo ""
    echo "🦀 Zero to Production in Rust"
    echo ""
    echo "Run 'menu' to see available commands."
    echo ""
  '';

  # ============================================
  # TESTS (run with `devenv test`)
  # ============================================
  enterTest = ''
    echo "Running devenv tests..."
    cargo --version
    cargo fmt -- --check
    cargo clippy -- -D warnings
    cargo check
    cog --version
  '';
}
