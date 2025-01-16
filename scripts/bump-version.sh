#!/bin/bash

# 检查是否提供了版本号参数
if [ -z "$1" ]; then
    echo "请提供版本号，例如: ./scripts/bump-version.sh 1.0.8"
    exit 1
fi

NEW_VERSION=$1

# 检测操作系统类型并使用对应的 sed 命令
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" package.json
    sed -i '' "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json
else
    # Linux 和其他系统
    sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" package.json
    sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json
fi

# 检查文件是否成功更新
if ! grep -q "\"version\": \"$NEW_VERSION\"" package.json || ! grep -q "\"version\": \"$NEW_VERSION\"" src-tauri/tauri.conf.json; then
    echo "❌ 版本更新失败，请检查文件内容"
    exit 1
fi

# 创建 git commit
git add package.json src-tauri/tauri.conf.json
git commit -m "chore: bump version to $NEW_VERSION"

# 创建 git tag
git tag "v$NEW_VERSION"

echo "✅ 版本更新完成！"
echo "现在你可以运行以下命令推送更改："
echo "git push && git push --tags" 