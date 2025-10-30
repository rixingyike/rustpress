# RustPress 构建指南

## 构建命令说明

RustPress 现在提供了灵活的构建选项，可以根据不同场景选择合适的构建方式。

### 1. 生产环境构建（推荐）

```bash
# 只构建 Markdown 文件，不编译 CSS
cargo run -- build
```

**特点：**
- ✅ 快速构建，只处理 Markdown 文件
- ✅ 适合生产环境部署
- ✅ 使用预编译的 CSS 文件
- ✅ 不需要 Node.js 环境

### 2. 开发环境构建

```bash
# 开发环境构建：先编译 CSS，再构建网站
cargo run -- build-dev
```

**特点：**
- 🔧 自动编译主题 CSS
- 🔧 构建完整网站
- 🔧 适合开发环境
- 🔧 需要 Node.js 环境

### 3. CSS 单独构建

```bash
# 单独编译主题 CSS
cargo run -- build-css
```

**使用场景：**
- 🎨 修改了 Tailwind 配置
- 🎨 更新了主题样式
- 🎨 需要重新生成 CSS

### 4. 开发模式（构建+服务器）

```bash
# 开发模式：构建 CSS + 构建网站 + 启动服务器
cargo run -- dev
```

**特点：**
- 🚀 自动构建 CSS
- 🚀 构建网站
- 🚀 启动本地服务器
- 🚀 适合开发调试

### 5. 预览模式

```bash
# 构建网站并启动服务器（使用现有 CSS）
cargo run -- serve
```

## 工作流程建议

### 主题开发者工作流程

1. **开发阶段**：
   ```bash
   # 方式1：使用开发模式（推荐）
   cargo run -- dev
   
   # 方式2：手动分步操作
   cd src/themes/default
   npm run dev  # 监听模式
   # 在另一个终端
   cargo run -- serve
   ```

2. **发布阶段**：
   ```bash
   # 构建生产版本的 CSS
   cargo run -- build-css
   
   # 提交代码（包括编译后的 CSS）
   git add .
   git commit -m "更新主题样式"
   ```

### 最终用户工作流程

1. **生产部署**（推荐）：
   ```bash
   # 克隆项目后直接构建（快速）
   cargo run -- build
   ```

2. **开发调试**：
   ```bash
   # 完整开发环境构建
   cargo run -- build-dev
   
   # 或者启动开发服务器
   cargo run -- dev
   ```

3. **自定义样式**：
   ```bash
   # 修改 tailwind.config.js 后重新构建 CSS
   cargo run -- build-css
   
   # 然后构建网站
   cargo run -- build
   ```

## 目录结构说明

```
rustpress/
├── src/themes/default/
│   ├── package.json          # 主题依赖（存在时自动检测需要 CSS 编译）
│   ├── tailwind.config.js    # Tailwind 配置
│   ├── src/tailwind.input.css # CSS 源文件
│   └── static/css/
│       ├── tailwind.css      # 编译后的 CSS（提交到仓库）
│       └── main.css          # 最终样式文件
├── source/                   # Markdown 文章目录
├── public/                   # 生成的静态网站
└── config.toml              # 站点配置
```

## 性能优化

- **生产构建**：`cargo run -- build` 只处理 Markdown，速度最快
- **CSS 缓存**：编译后的 CSS 文件被提交到仓库，避免重复编译
- **按需构建**：只有在样式修改时才需要重新编译 CSS

## 故障排除

### 1. CSS 构建失败
```bash
# 检查 Node.js 和 npm 是否安装
node --version
npm --version

# 重新安装依赖
cd src/themes/default
rm -rf node_modules package-lock.json
npm install
```

### 2. 主题不需要 CSS 编译
如果主题目录下没有 `package.json` 文件，系统会自动跳过 CSS 编译步骤。

### 3. 开发模式问题
```bash
# 如果开发模式启动失败，可以分步执行
cargo run -- build-css  # 先构建 CSS
cargo run -- serve      # 再启动服务器