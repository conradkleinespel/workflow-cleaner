# Workflow Cleaner

`workflow-cleaner` cleans up Github Actions workflow runs to preseve your privacy.

## Installation

To install Workflow Cleaner, you can run the following:
```shell
cargo install --git https://github.com/conradkleinespel/workflow-cleaner
```

## Usage

```shell
# Create a Github token at https://github.com/settings/personal-access-tokens
# with the following permissions:
# - Read access to metadata
# - Read and Write access to actions
export GITHUB_TOKEN=xxx

# Remove workflows older than 30 days for all your repositories
workflow-cleaner

# View options
workflow-cleaner --help
```

## License

The source code is released under the Apache 2.0 license.
