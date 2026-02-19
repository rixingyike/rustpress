/**
 * RustPress 评论系统 GitHub OAuth 代理服务 (Cloudflare Worker)
 * 
 * =========================================================================
 * 使用说明：
 * =========================================================================
 * 
 * 1. 部署环境：
 *    - 登录 Cloudflare Dashboard -> Workers & Pages -> Create Application -> Create Worker
 *    - 将本文件内容复制到 Worker 编辑器中 (通常是 worker.js)
 *    - 点击 "Save and Deploy"
 * 
 * 2. 环境变量配置 (Settings -> Variables -> Add Variable)：
 *    - GITHUB_CLIENT_ID:     <您的 GitHub App Client ID>
 *    - GITHUB_CLIENT_SECRET: <您的 GitHub App Client Secret> (建议点击 Encrypt 加密)
 *    - BLOG_DOMAIN:          <您的博客域名> (例如 https://yishulun.com)
 * 
 * 3. GitHub App 配置：
 *    - 登录 GitHub -> Settings -> Developer settings -> OAuth Apps
 *    - 找到您的 Blog App
 *    - 将 "Authorization callback URL" 修改为您的 Worker 地址：
 *      https://<您的-worker-名>.<您的-subdomain>.workers.dev/callback
 *      (或者您绑定的自定义域名/callback)
 * 
 * 4. 博客侧配置：
 *    - 确保 rustpress 的 themes/default/templates/comments.html 中生成的登录链接
 *      不要携带 redirect_uri 参数（或者携带指向本 Worker 的 redirect_uri）
 *    - 确保 state 参数包含当前页面的路径
 * 
 * =========================================================================
 */
export const name = "rustpress-comments";

export default {
    async fetch(request, env, ctx) {
        // 配置：您的博客域名 (从环境变量读取，增强安全性)
        // 作用：验证登录后跳转的目标地址是否属于您的博客，防止开放重定向漏洞
        // 如果环境变量未配置，默认使用 https://yishulun.com
        const BLOG_DOMAIN = env.BLOG_DOMAIN || "https://yishulun.com";

        const url = new URL(request.url);

        // 路由：/callback (GitHub 授权后回调此接口)
        if (url.pathname === "/callback") {
            const code = url.searchParams.get("code");
            const state = url.searchParams.get("state") || "/"; // 原始页面路径

            // 如果没有 code 参数，说明不是合法的回调
            if (!code) {
                return new Response("Missing code parameter", { status: 400 });
            }

            try {
                // 向 GitHub 请求交换 Token (Client Secret 安全地保存在 Worker 后端)
                const response = await fetch("https://github.com/login/oauth/access_token", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                        "Accept": "application/json",
                    },
                    body: JSON.stringify({
                        client_id: env.GITHUB_CLIENT_ID,
                        client_secret: env.GITHUB_CLIENT_SECRET,
                        code: code,
                    }),
                });

                const data = await response.json();

                if (data.error) {
                    return new Response(`GitHub Error: ${data.error_description}`, { status: 400 });
                }

                // 登录成功，准备跳转回博客页面
                // Token 将通过 URL 参数传递给前端

                // 安全检查：确保跳转目标符合预期
                // 简单的防范开放重定向攻击
                let targetPath = state;
                try {
                    // 如果 state 是一个完整的 URL
                    if (targetPath.startsWith("http")) {
                        const targetUrl = new URL(targetPath);
                        // 必须是 BLOG_DOMAIN 或者 localhost (方便本地调试)
                        if (targetUrl.origin !== BLOG_DOMAIN && !targetUrl.hostname.includes("localhost")) {
                            targetPath = "/"; // 不合法，回退到首页
                        } else {
                            // 合法，直接使用
                            targetPath = targetUrl.href;
                            const redirectUrl = `${targetPath}${targetPath.includes('?') ? '&' : '?'}gh_token=${data.access_token}`;
                            return Response.redirect(redirectUrl, 302);
                        }
                    } else if (!targetPath.startsWith("/")) {
                        // 如果只是路径但没有斜杠，补上
                        targetPath = "/" + targetPath;
                    }
                } catch (e) {
                    targetPath = "/";
                }

                // 构造最终跳转地址
                const redirectUrl = `${BLOG_DOMAIN}${targetPath}${targetPath.includes('?') ? '&' : '?'}gh_token=${data.access_token}`;

                return Response.redirect(redirectUrl, 302);

            } catch (e) {
                return new Response(`Server Error: ${e.message}`, { status: 500 });
            }
        }

        // 默认响应
        return new Response("RustPress OAuth Proxy is Running. Use /callback", { status: 200 });
    },
};
