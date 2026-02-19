export default {
    async fetch(request, env, ctx) {
        // Config: Your blog domain (User configurable)
        // This allows you to verify the redirect target is within your domain
        const BLOG_DOMAIN = "https://yishulun.com";

        const url = new URL(request.url);

        // Route: /callback (GitHub calls this)
        if (url.pathname === "/callback") {
            const code = url.searchParams.get("code");
            const state = url.searchParams.get("state") || "/"; // original page path

            if (!code) {
                return new Response("Missing code parameter", { status: 400 });
            }

            try {
                // Exchange code for token securely (Client Secret stays on server)
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

                // Direct Redirect to the original page (state) with token
                // The comments script on that page will pick it up.

                // Ensure state is a valid path to prevent open redirect vulnerabilities
                // Simple check: must start with / and not // (protocol relative)
                let targetPath = state;
                try {
                    // If state is a full URL, ensure it starts with BLOG_DOMAIN
                    if (targetPath.startsWith("http")) {
                        const targetUrl = new URL(targetPath);
                        if (targetUrl.origin !== BLOG_DOMAIN && !targetUrl.hostname.includes("localhost")) {
                            // Security: Prevent redirect to arbitrary domains
                            targetPath = "/";
                        } else {
                            // Use the full URL if valid
                            targetPath = targetUrl.href;
                            // We don't prepend BLOG_DOMAIN if it's already a full URL
                            const redirectUrl = `${targetPath}${targetPath.includes('?') ? '&' : '?'}gh_token=${data.access_token}`;
                            return Response.redirect(redirectUrl, 302);
                        }
                    } else if (!targetPath.startsWith("/")) {
                        targetPath = "/" + targetPath;
                    }
                } catch (e) { targetPath = "/"; }

                const redirectUrl = `${BLOG_DOMAIN}${targetPath}${targetPath.includes('?') ? '&' : '?'}gh_token=${data.access_token}`;

                return Response.redirect(redirectUrl, 302);

            } catch (e) {
                return new Response(`Server Error: ${e.message}`, { status: 500 });
            }
        }

        return new Response("RustPress OAuth Proxy is Running. Use /callback", { status: 200 });
    },
};
