# Properly and safely get the current version, update the version patch with 1, and commit, tag, push with the new version

# Get version from cargo.toml
$version = (Select-String -Path "Cargo.toml" -Pattern 'version = "(\d+\.\d+\.\d+)"' | ForEach-Object { $_.Matches.Groups[1].Value })

# Split version into major, minor, and patch
$versionParts = $version -split '\.'
$major = [int]$versionParts[0]
$minor = [int]$versionParts[1]
$patch = [int]$versionParts[2]

# Increment patch version
$newPatch = $patch + 1
$newVersion = "$major.$minor.$newPatch"

# Ensure the new version does not conflict with existing tags
$existingTags = git tag
if ($existingTags -contains $newVersion) {
    Write-Host "Version $newVersion already exists. Aborting."
    exit 1
}

# Update version in Cargo.toml
(Get-Content Cargo.toml) -replace "version = `"$version`"", "version = `"$newVersion`"" | Set-Content Cargo.toml

# Automatically commit all changes
git add .
git commit -m "Bump version to $newVersion"

# Tag and push
git tag -a $newVersion -m "version $newVersion"
git push origin $newVersion

# Cargo publish with --allow-dirty flag to include uncommitted changes
cargo publish --allow-dirty
