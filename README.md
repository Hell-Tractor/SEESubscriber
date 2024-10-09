# SEESubscriber

同济大学电子与信息工程学院通知/公告与同济大讲堂订阅系统

本项目仅供学习交流使用，请勿利用本项目影响学校服务器正常运行。

## 使用说明

1. 从[Release](https://github.com/Hell-Tractor/SEESubscriber/releases)页面下载最新版本的可执行文件。
2. 创建config.yaml文件（或从release页面下载）。并将其放置在可执行文件同一目录下。
3. 按需修改config.yaml文件中的配置。
4. 配置环境变量
5. 运行可执行文件。

## 配置文件说明

- `url`: 通知/公告页面的URL，参考[config.yaml](config.yaml)。一般情况下无需修改。
- `pages`: 需要订阅的页面。请访问[通知公告](http://see.tongji.edu.cn/notice)页面，自行查看并修改需要订阅的页面
- `notice`: 需要的通知发送方式。目前支持`sct`、`sc3`和`local`三种方式。分别为Server酱、Server酱 $^3$ 推送和本地通知。`sct`与`sc3`方式需要配置对应的环境变量，见下文。
- `lecture_url`: 获取同济大讲堂的URL。一般情况下无需修改。
- `report_error`: 当程序执行失败时，是否通过`notice`定义的渠道发送错误消息。

## 环境变量

- `RUST_LOG`: 控制日志输出等级。
- `SEE_SCT_KEY`: Server酱的SCT_KEY，用于推送通知到微信。参考[Server酱](https://sct.ftqq.com/)。
- `SEE_SC3_KEY`: Server酱 $^3$ 的SC3_KEY，用于推送通知到APP。参考[Server酱 $^3$ ](https://sc3.ft07.com/)。
- `SEE_LOGIN_USERNAME`: 用于统一身份验证的用户名
- `SEE_LOGIN_PASSWORD`: 用于统一身份验证的密码

## 安全性说明

本程序不会保存你的用户名和密码，用户名和密码只会通过环境变量的形式保存在内存中。但程序会将登录后的`sessionid`缓存在`data.json`中，对应的key为`sessionid`，可以手动删除，不影响程序正常运行。

本程序在登录后仅会访问获取同济大讲堂列表的API，不会将`sessionid`用作其他用途。也不会向学校网站发起高频次访问。