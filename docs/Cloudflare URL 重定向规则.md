# Cloudflare URL 重定向规则

## 背景

架构调整后，网站 URL 结构发生了以下变化：

| 旧 URL | 新 URL | 说明 |
|--------|--------|------|
| `/posts/2008/01.html` | `/2008/01.html` | 博客文章从 `posts/` 移至根级年份目录 |
| `/posts/2008/*` | `/2008/*` | 批量重定向所有 posts 旧链接 |
| `/blog/2026/1.html` | `/2026/1.html` | blog 前缀已移除（注：旧版 rustpress 已自动剥离 blog 前缀，此规则为防护性兜底） |
| `/categories.html` | `/categories/` | 分类概览页改用目录式路径（实际渲染为 `/categories/index.html`） |
| `/tags.html` | `/tags/` | 标签云概览页改用目录式路径（实际渲染为 `/tags/index.html`） |
| `/archives.html` | `/archives/` | 归档概览页改用目录式路径（实际渲染为 `/archives/index.html`） |
| `/archives/:year.html` | `/archives/:year/` | 年份归档页改用目录式路径（实际渲染为 `/archives/:year/index.html`） |

需要说明的是，旧版 rustpress 在编译时已将 `source/blog/` 下的 `blog/` 前缀自动剥离，因此 `/blog/2026/1.html` 这类 URL 本身就不存在，设置此规则仅为防止未来可能的路由回归或第三方引用了错误的 URL。此外，本次架构调整统一将所有列表页改写为目录结构（无后缀化/统一使用 index.html 挂载）。

## Cloudflare 配置方式

### 方式一：Bulk Redirect（推荐）

Bulk Redirect 不消耗请求额度，性能最佳。

1. 登录 Cloudflare Dashboard
2. 进入域名（如 `yishulun.com`）→ **Rules** → **Bulk Redirects**
3. 创建一个新的 Bulk Redirect List：

| Source URL | Target URL | Status |
|-----------|-----------|--------|
| `yishulun.com/posts/2008/01.html` | `yishulun.com/2008/01.html` | 301 |
| `yishulun.com/posts/2008/02.html` | `yishulun.com/2008/02.html` | 301 |
| `yishulun.com/posts/2008/03.html` | `yishulun.com/2008/03.html` | 301 |
| `yishulun.com/posts/2008/04.html` | `yishulun.com/2008/04.html` | 301 |
| `yishulun.com/posts/2008/05.html` | `yishulun.com/2008/05.html` | 301 |
| `yishulun.com/posts/2008/06.html` | `yishulun.com/2008/06.html` | 301 |

4. 添加一条 Bulk Redirect Rule，将此 List 关联并启用 **Preserve query string**、**Subpath matching**。

### 方式二：使用通配符的 Page Rule（最简单）

1. 登录 Cloudflare Dashboard
2. 进入域名 → **Rules** → **Page Rules**
3. 创建一条 Page Rule：

   - **If URL matches**: `yishulun.com/posts/2008/*`
   - **Setting**: **Forwarding URL** (301)
   - **Destination URL**: `https://yishulun.com/2008/*`

4. 保存即可。

### 方式三：通过 `_redirects` 文件（兼容 Cloudflare Pages）

如果站点部署在 Cloudflare Pages 上，可以在项目根目录或 `public/` 目录放置 `_redirects` 文件：

```
# /public/_redirects
/posts/2008/01.html  /2008/01.html  301
/posts/2008/02.html  /2008/02.html  301
/posts/2008/03.html  /2008/03.html  301
/posts/2008/04.html  /2008/04.html  301
/posts/2008/05.html  /2008/05.html  301
/posts/2008/06.html  /2008/06.html  301
/posts/*             /2008/:splat   301
/categories.html     /categories/   301
/tags.html           /tags/         301
/archives.html       /archives/     301
/archives/:year.html /archives/:year/ 301
```

### 方式四：通过 `wrangler.toml` + Workers（适用于 Cloudflare Workers 部署）

在 `wrangler.toml` 中添加路由配置，或在 Worker 脚本中使用 `Response.redirect`：

```js
// wrangler.toml 中无需额外配置，直接在 Worker 中处理
export default {
  async fetch(request) {
    const url = new URL(request.url);
    const path = url.pathname;

    // posts/ 重定向
    if (path.startsWith('/posts/')) {
      const newPath = path.replace('/posts/', '/2008/');
      return Response.redirect(new URL(newPath, request.url), 301);
    }

    // blog/ 重定向（防护性兜底）
    if (path.startsWith('/blog/')) {
      const newPath = path.replace('/blog/', '/');
      return Response.redirect(new URL(newPath, request.url), 301);
    }

    return fetch(request);
  }
}
```

## 重定向映射表

### 核心重定向

| 旧路径 | 新路径 | 说明 |
|--------|--------|------|
| `/posts/2008/01.html` | `/2008/01.html` | 第一篇旧博文 |
| `/posts/2008/02.html` | `/2008/02.html` | 第二篇旧博文 |
| `/posts/2008/03.html` | `/2008/03.html` | 第三篇旧博文 |
| `/posts/2008/04.html` | `/2008/04.html` | 第四篇旧博文 |
| `/posts/2008/05.html` | `/2008/05.html` | 第五篇旧博文 |
| `/posts/2008/06.html` | `/2008/06.html` | 第六篇旧博文 |
| `/categories.html` | `/categories/` | 分类列表页 |
| `/tags.html` | `/tags/` | 标签云列表页 |
| `/archives.html` | `/archives/` | 归档列表页 |
| `/archives/:year.html` | `/archives/:year/` | 年份归档页面 |

### 通配规则

| 匹配模式 | 目标模式 | 说明 |
|---------|---------|------|
| `/posts/2008/*` | `/2008/*` | 全部旧 posts 重定向 |
| `/posts/*` | `/2008/:splat` | 泛匹配（按年份自动映射） |
| `/blog/*` | `/:splat` | blog 前缀移除（防护性兜底） |
| `/archives/:year.html` | `/archives/:year/` | 年份归档页重定向 |

## 验证方法

配置完成后，用 curl 验证重定向是否生效：

```bash
# 测试单个页面重定向
curl -I https://yishulun.com/posts/2008/01.html

# 期望输出：
# HTTP/2 301
# location: https://yishulun.com/2008/01.html

# 测试通配重定向
curl -I https://yishulun.com/posts/2008/06.html
# 期望输出：
# HTTP/2 301
# location: https://yishulun.com/2008/06.html

# 测试 blog 兜底
curl -I https://yishulun.com/blog/2026/1.html
# 期望输出：
# HTTP/2 301
# location: https://yishulun.com/2026/1.html
```

## 注意事项

1. **SEO**：使用 301（永久重定向）而非 302，确保搜索引擎将旧链接的权重转移到新链接。
2. **缓存**：Cloudflare 会自动缓存 301 重定向，配置后可能需要等待几分钟全球生效。
3. **通配符优先级**：Page Rule 中的通配符匹配遵循最具体优先原则，如果同时存在具体路径和通配路径，具体路径优先匹配。
4. **Bulk Redirect vs Page Rule**：Bulk Redirect 免费额度更高（每个 zone 可配 10 条规则），Page Rule 免费版仅 3 条。
