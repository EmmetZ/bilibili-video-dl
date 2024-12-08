**简易的命令行 bilibili 视频下载工具**

![example](./example/demo.gif)

```sh
Usage: bili-dl [OPTIONS] <url> [path]

Arguments:
  <url>   视频/番剧链接
  [path]  下载目录路径 [default: ./]

Options:
  -c, --cookies <path>  cookies.txt 的路径
  -h, --help            Print help
  -V, --version         Print version
```

支持下载普通视频、部分番剧。会员内容需要传入 Cookie

`cookies.txt` 示例:
```txt
SESSDATA=XXX; .bilibili.com
```

**参考**

[哔哩哔哩-API收集整理](https://socialsisteryi.github.io/bilibili-API-collect/)
