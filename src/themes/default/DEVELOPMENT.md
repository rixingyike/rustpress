# 开发指南

## 构建策略说明

本主题采用混合构建策略，兼顾开发便利性和用户友好性：

### 对于主题作者（开发者）

1. **开发模式**：
   ```bash
   npm run dev
   ```
   - 生成 `tailwind.dev.css`（被 gitignore 忽略）
   - 用于开发时的实时预览

2. **发布模式**：
   ```bash
   npm run release
   ```
   - 生成压缩的 `tailwind.css`（提交到仓库）
   - 供最终用户直接使用

### 对于最终用户

有两种使用方式：

#### 方式1：直接使用（推荐给普通用户）
- 直接使用仓库中已编译的 `static/css/tailwind.css`
- 无需安装 Node.js 或运行构建命令
- 适合只想使用主题的用户

#### 方式2：自定义构建（推荐给开发者）
```bash
cd src/themes/default
npm install
npm run build-css
```
- 可以修改 `tailwind.config.js` 自定义样式
- 适合需要定制主题的用户

## 文件说明

- `src/tailwind.input.css` - Tailwind 源文件
- `static/css/tailwind.css` - 编译后的生产版本（提交到仓库）
- `static/css/tailwind.dev.css` - 开发版本（被忽略）
- `static/css/main.css` - 最终样式文件（导入 tailwind.css）

## 工作流程

### 主题开发者工作流程
1. 修改 `src/tailwind.input.css` 或 `tailwind.config.js`
2. 运行 `npm run dev` 进行开发
3. 开发完成后运行 `npm run release` 生成生产版本
4. 提交代码（包括编译后的 `tailwind.css`）

### 用户使用流程
1. 克隆仓库
2. 直接使用，或者运行 `npm install && npm run build-css` 进行自定义

这种策略的优势：
- ✅ 降低用户使用门槛
- ✅ 保持开发灵活性
- ✅ 避免开发时的文件冲突
- ✅ 支持用户自定义构建