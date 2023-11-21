# 官方收费加算法更新此项目已废 仅供参考



[如需使用alist-encrypt需要设置webdav的用户名和密码，详情见命令行]
# webdav服务 目前支持列文件及通过curl上传【需要带checksum的header否则会报错】  
例子
```
curl -T "文件名" "http://127.0.0.1:9867/"  --header 'OC-Checksum:sha1:文件名的sha1'
```  

Docker主页: https://hub.docker.com/r/ykxvk8yl5l/stariver-webdav   

# 使用方法 【token可通过后台获取】
1、命令行
```
stariver-webdav --stariver-token='XXXXXXXXXXXXX' --auth-user='admin' --auth-password='admin' 
```
2、Dokcer【推荐使用，如不使用alist-encrypt可不设置用户名和密码，数据持久化目录为/root】
```
docker run  --name="stariver-webdav" -p 10020:9867 -e STARIVER_TOKEN="XXXXXXXXXXXXX" -e WEBDAV_AUTH_USER="admin" -e WEBDAV_AUTH_PASSWORD="admin" ykxvk8yl5l/stariver-webdav:latest
```
