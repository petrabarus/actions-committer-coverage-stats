# Committers Code Coverage Stats
Github action to analyze the code coverage of the committers in a repository.

## 1. Inputs

### 1.1 `min_threshold`

The minimum threshold for the committers' code coverage percentage. If the code coverage percentage of a committer is below this threshold, the committer will be considered as a failing committer.

Default: `80`

### 1.2 `coverage_files`

The list of coverage files to be analyzed. The coverage files should be in the format of `path/to/coverage.xml`. Multiple coverage files can be provided by separating them with a comma.

## 2. Outputs

## 3. Example Usage

```yaml
```

## 4. Troubleshooting

## 5. License

[BSD 2-Clause License](https://opensource.org/license/bsd-2-clause)