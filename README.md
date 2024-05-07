# Linux登录校园网

![校园网登录界面](D:\code\C\curl\campusnet\src\src20240504\校园网登录界面.png)

## c语言版使用方法

1. 编译 make

2. 参数设置

       -h, --help          give you some information to better use this program
       -V, --version       get this program version
       -u, --username      your campus card ID
       -p, --password      your campus card password
       -s, --service       your campus network operator: cmcc or unicom or ctcc or default or local

3. 登录

   ```
   ./login -u campuscardid -p campuscardpwd -s cmcc
   ```

## python版使用方法（pythonsrc20240507）

1. 环境
   `python 3.10.0`

   库
   `pycurl`

2. 使用方法

   ```
   python main.py -u campuscardid -p campuscardpwd -s cmcc
   ```

​	c语言在跨平台的使用上确实不方便所以开发了python版，程序里增加了自定义`url`，如果你想试试自己的校园网能否使用修改试试。

## 结果

1. 登录成功

   ```
   username : campuscardid
   service : cmcc
   NOTICE:
       success
   ```

2. 登录失败 

   ```
   username : campuscardid
   service : cmcc
   NOTICE:
       fail
       WEB认证设备未注册，请确认SAM+/portal/设备上的参数配置是否一致
   ```

   ```
   username : campuscardid
   service : cmcc
   NOTICE:
       fail
       用户不存在或者密码错误!
   ```

   ```
   username : campuscardid
   service : ctcc
   NOTICE:
       fail
       您未绑定服务对应的运营商!
   ```

​	

## 缺憾

1. 没有检测是否已经登录成功。
   如果使用定时检测是否链接互联网，可以

   ```
   ping www.baidu.com
   ```

   或者`ping`其他地址。

2. 目前代码只支持中国石油大学（华东）。
   但是应该其他锐捷的登录也是可以实现的，如果你有什么问题可以联系我。
   `Email：tianyuyang666@gmail.com`


