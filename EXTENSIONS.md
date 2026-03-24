# Pact CLI Extensions

The Pact CLI supports an extension system that allows you to install and manage additional tools and legacy binaries. Extensions integrate with the main CLI, providing access to functionality like AI-powered contract testing and legacy Pact Ruby based tools.

## 🚀 Quick Start

```bash
# List available extensions
pact extension list

# Install PactFlow AI extension
pact extension install pactflow-ai

# Install Ruby legacy tools
pact extension install pact-legacy

# Use PactFlow AI directly
pact pactflow ai --help

# Use legacy tools
pact extension pact-broker-legacy --help
```

## 📦 Available Extensions

### PactFlow AI

**AI-augmented contract testing tools**

- **Name**: `pactflow-ai`
- **Type**: PactFlow AI
- **Usage**: `pact pactflow ai <args>` or `pact extension pactflow-ai <args>`
- **Features**: AI-augmented contract generation, reviews
- **Platforms**: macOS (Intel/Apple Silicon), Linux (x64/ARM64), Windows (x64/ARM64)

### Drift

**Contract conformance testing — validate whether an API implementation matches its published specification.**

- **Name**: `drift`
- **Type**: Drift
- **Usage**: `pact drift <args>` or `pact extension drift <args>`
- **Features**: Contract conformance testing, API implementation validation
- **Platforms**: macOS (Intel/Apple Silicon), Linux (x64/ARM64), Windows (x64)

Drift is installed to `~/.drift/` by default:

### Ruby Legacy Tools

**Traditional Ruby-based Pact tools (pact-legacy)**

- **Name**: `pact-legacy`
- **Type**: Ruby Legacy
- **Includes**:
  - `pact-broker-legacy` - Legacy Pact Broker client
  - `pactflow-legacy` - Legacy PactFlow client
  - `message-legacy` - Legacy message pact tools
  - `mock-legacy` - Legacy mock service
  - `verifier-legacy` - Legacy provider verifier
  - `stub-legacy` - Legacy stub service
- **Usage**: `pact extension <tool-name> <args>`
- **Platforms**: macOS, Linux, Windows

## 🛠 Extension Management

### Installation

```bash
# Install latest version
pact extension install pactflow-ai
pact extension install pact-legacy

# Install specific version
pact extension install pactflow-ai --version 1.11.4
pact extension install pact-legacy --version v2.5.5
```

### Listing Extensions

```bash
# List all extensions
pact extension list

# Show only installed extensions
pact extension list --installed
```

Output example:

```
📦 Available extensions:
Name                 Type       Installed       Latest          Status    
---------------------------------------------------------------------------
pactflow-ai          PactFlow AI 1.11.4         1.11.4          ✅ Installed
pact-broker-legacy   Ruby Legacy v2.5.5         v2.5.5          ✅ Installed
pactflow-legacy      Ruby Legacy v2.5.5         v2.5.5          ✅ Installed
message-legacy       Ruby Legacy v2.5.5         v2.5.5          ✅ Installed
mock-legacy          Ruby Legacy v2.5.5         v2.5.5          ✅ Installed
verifier-legacy      Ruby Legacy v2.5.5         v2.5.5          ✅ Installed
stub-legacy          Ruby Legacy v2.5.5         v2.5.5          ✅ Installed
```

### Updating Extensions

```bash
# Update all installed extensions
pact extension update --all

# Update specific extension
pact extension update pactflow-ai
pact extension update pact-legacy
```

### Uninstalling Extensions

```bash
# Uninstall PactFlow AI
pact extension uninstall pactflow-ai

# Uninstall Ruby legacy tools (removes all legacy tools)
pact extension uninstall pact-legacy

# Uninstall individual legacy tool
pact extension uninstall pact-broker-legacy
```

## 🔧 Usage Patterns

### PactFlow AI Integration

The PactFlow AI extension integrates seamlessly with the `pactflow` command:

```bash
# These are equivalent:
pact pactflow ai --help
pact extension pactflow-ai --help
```

### Legacy Ruby Tools

Legacy tools are accessed via the extension system:

```bash
# Legacy Pact Broker operations
pact extension pact-broker-legacy can-i-deploy --pacticipant my-app --version 1.0.0

# Legacy provider verification
pact extension verifier-legacy

# Legacy mock service
pact extension mock-legacy
```

## 📁 Extension Storage

Extensions are installed to `~/.pact/extensions/` by default:

```
~/.pact/extensions/
├── config.json                    # Extension configuration
├── bin/                           # Symlinks to extension binaries
│   ├── pactflow-ai
│   ├── pact-broker-legacy
│   ├── pactflow-legacy
│   ├── message-legacy
│   ├── mock-legacy
│   ├── verifier-legacy
│   └── stub-legacy
└── pact-legacy/               # Ruby tools installation
    └── bin/
        ├── pact-broker
        ├── pactflow
        ├── pact-message
        ├── pact-mock-service
        ├── pact-provider-verifier
        └── pact-stub-service
```

### Custom Installation Directory

You can customize the extension directory using the `PACT_CLI_EXTENSIONS_HOME` environment variable:

```bash
export PACT_CLI_EXTENSIONS_HOME=/opt/pact-extensions
pact extension install pactflow-ai
```

## 🔍 Version Management

The extension system provides intelligent version tracking:

### PactFlow AI Versions

- **Installed Version**: Retrieved by executing `pactflow-ai --version`
- **Latest Version**: Fetched from `https://download.pactflow.io/ai/dist/{platform}/latest`
- **Manual Updates**: Use `pact extension update pactflow-ai`

### Ruby Legacy Versions

- **Installed Version**: Recorded during installation from GitHub release tag
- **Latest Version**: Fetched from GitHub API `/repos/pact-foundation/pact-standalone/releases/latest`
- **Manual Updates**: Use `pact extension update pact-legacy`

## 🌍 Platform Support

### PactFlow AI

- **macOS**: Intel (x86_64) and Apple Silicon (aarch64)
- **Linux**: x86_64 and aarch64
- **Windows**: x86_64 and aarch64

### Ruby Legacy Tools

- **macOS**: Intel (x86_64) and Apple Silicon (aarch64)
- **Linux**: x86_64 and aarch64
- **Windows**: x86_64 only

## 🔗 Integration Examples

### GitHub Actions

```yaml
# GitHub Actions example
- uses: `pact-foundation`/pact-cli@main
- name: Install Pact Extensions
  run: |
    pact extension install pactflow-ai
    pact extension install pact-legacy
- name: Or Install All Pact Extensions
  run: |
    pact extension install --all
```
