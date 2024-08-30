# SEESubscriber

同济大学电子与信息工程学院通知/公告订阅系统

本项目仅供学习交流使用，请勿利用本项目影响学校服务器正常运行。

## 使用说明

1. 从[Release](https://github.com/Hell-Tractor/SEESubscriber/releases)页面下载最新版本的可执行文件。
2. 创建config.yaml文件（或从项目根目录下载）。并将其放置在可执行文件同一目录下。
3. 按需修改config.yaml文件中的配置。
4. 配置环境变量
5. 运行可执行文件。

## 配置文件说明

- `url`: 通知/公告页面的URL，参考[config.yaml](config.yaml)。一般情况下无需修改。
- `pages`: 需要订阅的页面。请访问[通知公告](http://see.tongji.edu.cn/notice)页面，自行查看并修改需要订阅的页面
- `notice`: 需要的通知发送方式。目前支持`sct`和`local`两种方式。分别为Server酱推送和本地通知。`sct`方式需要配置`NOTICE_SCT_KEY`环境变量。

## 环境变量

- `RUST_LOG`: 控制日志输出等级。
- `NOTICE_SCT_KEY`: Server酱的SCT_KEY，用于推送通知到微信。参考[Server酱](https://sct.ftqq.com/)。