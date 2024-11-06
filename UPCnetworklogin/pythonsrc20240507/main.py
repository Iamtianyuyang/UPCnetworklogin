# UPC 2024.05.08
import get_opt
import my_info
import function

if __name__ == '__main__':
    my = my_info.info(0, 0, 0)
    my = get_opt.get_opt(my)
    print(my.username)
    print(my.service)
    url = "lan.upc.edu.cn"
    queryString = function.get_url(url)
    queryString = function.transcode(queryString)
    function.login(url, queryString, my)




