# RustPress 默认主题

这是 RustPress 的默认主题，使用 Tailwind CSS 构建。

## 开发环境设置

### 1. 安装依赖

```bash
cd src/themes/default
npm install
```

### 2. 开发模式

```bash
# 监听 CSS 文件变化并自动编译
npm run dev
```

### 3. 生产构建

```bash
# 构建压缩后的 CSS 文件
npm run build-css
```

## 目录结构

```
src/themes/default/
├── package.json              # 主题依赖管理
├── tailwind.config.js        # Tailwind 配置
├── postcss.config.js         # PostCSS 配置
├── src/
│   ├── tailwind.input.css    # Tailwind 源文件
│   └── components/           # 自定义组件样式
├── static/
│   ├── css/
│   │   ├── main.css          # 最终样式文件
│   │   └── tailwind.css      # Tailwind 编译输出
│   └── js/
└── templates/                # 模板文件
```

## 自定义样式

### 1. 修改主题配置

编辑 `tailwind.config.js` 来自定义颜色、字体等。

### 2. 添加组件样式

在 `src/tailwind.input.css` 的 `@layer components` 中添加自定义组件。

### 3. 添加工具类

在 `@layer utilities` 中添加自定义工具类。

## 开发工作流

1. 修改 `src/tailwind.input.css` 或 `tailwind.config.js`
2. 运行 `npm run dev` 开始监听
3. 在模板中使用 Tailwind 类名
4. 生产部署前运行 `npm run build-css`

## 注意事项

- 不要直接编辑 `static/css/tailwind.css`，它是自动生成的
- 自定义样式应该添加到 `src/tailwind.input.css` 中
- 生产环境记得运行构建命令以获得最小化的 CSS