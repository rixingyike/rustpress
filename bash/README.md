# bash
一些通用脚本，在个人电脑上使用的

## 设置脚本

Git Bash 是基于 MSYS2（Minimal SYStem 2）的简化版 Unix-like 环境，默认使用 Bash 作为交互式 Shell。它会遵循标准的 Bash 启动文件加载顺序，但针对 Windows 环境做了适配：

```
文件	    作用
~/.bash_profile	登录 Shell 的全局配置文件（Git Bash 会优先检查并生成此文件）
~/.bashrc	交互式非登录 Shell 的配置文件（用户自定义别名、函数等）
```

Git Bash会自动加载bash_profile，并间接加载bashrc配置。


```
cp -rf scripts/.*.sh ~/
cp ./.bashrc ~/
chmod +x $HOME/.*.sh
```

设置git

```
git config --global user.email "9830131@qq.com"
git config --global user.name "李艺"
```

