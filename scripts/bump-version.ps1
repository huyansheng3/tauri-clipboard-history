param(
    [Parameter(Mandatory=$true)]
    [string]$NewVersion
)

# 更新 package.json
$packageJson = Get-Content package.json -Raw
$packageJson = $packageJson -replace '"version": ".*"', "`"version`": `"$NewVersion`""
Set-Content package.json $packageJson

# 更新 tauri.conf.json
$tauriConf = Get-Content src-tauri/tauri.conf.json -Raw
$tauriConf = $tauriConf -replace '"version": ".*"', "`"version`": `"$NewVersion`""
Set-Content src-tauri/tauri.conf.json $tauriConf

# 创建 git commit
git add package.json src-tauri/tauri.conf.json
git commit -m "chore: bump version to $NewVersion"

# 创建 git tag
git tag "v$NewVersion"

Write-Host "✅ 版本更新完成！"
Write-Host "现在你可以运行以下命令推送更改："
Write-Host "git push && git push --tags" 