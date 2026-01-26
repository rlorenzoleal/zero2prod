# Zero to Production in Rust

A production-ready email newsletter API built following the book [Zero to Production in Rust](https://www.zero2prod.com/) by Luca Palmieri.

## ⚠️ Note

This is a personal learning project. Issues and external PRs are not accepted.
Feel free to fork it for your own learning!

## Prerequisites

- [Nix](https://nixos.org/download.html) package manager
- [devenv](https://devenv.sh/getting-started/)

### Installing Nix
```bash
# Linux/macOS (recommended: Determinate Systems installer)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

### Installing devenv
```bash
nix profile install github:cachix/devenv/latest
```

## Getting Started

1. Clone the repository:
```bash
   git clone https://github.com/YOUR_USERNAME/zero2prod.git
   cd zero2prod
```

2. Enter the development environment:
```bash
   devenv shell
```
   
   First run will take a few minutes to download and build dependencies.

3. See available commands:
```bash
   menu
```

## Development Commands

| Command | Description |
|---------|-------------|
| `dev:watch` | Watch for changes and run check → test → run |
| `dev:test` | Run tests with nextest |
| `dev:coverage` | Generate code coverage report |
| `dev:lint` | Run clippy with strict warnings |
| `dev:fmt` | Format code |
| `dev:audit` | Check for security vulnerabilities |

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/). Use the `commit` helper:
```bash
commit feat "add user registration"
commit fix "handle empty email" validation
```

## License

MIT