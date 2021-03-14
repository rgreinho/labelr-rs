# labelr-rs

Manage your GitHub labels efficiently.

With `labelr`, managing your GitHub labels becomes effortless. `labelr` will attempt to detect all the information required to apply the labels wherever you need them to be.

## Infered values and environment variables

`labelr` will automatically detect the owner or organization and the repostiory from the directory where you are running the command. It will also look automatically for a file named `labels.yml`.

The following environment variables are used by `labelr`:

* GITHUB_ORGANIZATION
* GITHUB_REPOSITORY
* GITHUB_USER
* GITHUB_TOKEN

### Precedence

`labelr` looks for information in this order:

1. Infered information from current directory
2. environment variables
3. CLI arguments

## Existing labels

For existing labels, description and color will be updated to match the content of `the labels.yml` file.

However, **labels cannot be renamed**. This is due to the fact that the tool does not keep track of the existing configuration. If the name of a label gets changed, a new label will be created.

## labels.yml

The `labels.yml` file has a simple format:

```yml
---
labels:
  - name: "kind/bug"
    color: "#D73A4A"
    description: "Something isn't working"
```

The top level key `labels` is used to group the labels together. Each label then becomes an entry under this key.

Each label entry is composed of the following fields:

* `name` (required)
* `color` (required)
* `description` (optional)

For a complete example, have a look at the labels used
[for this project](https://github.com/rgreinho/labelr/blob/master/.github/labels.yml).
