import re
import sys
import json
import pycurl


def check_full_info(my):
    if my.username == 0 or my.password == 0 or my.service == 0:
        print("WARNING:")
    if my.username == 0:
        print("    please enter you username. -u or --username")
    if my.password == 0:
        print("    please enter you password. -p or --password")
    if my.service == 0:
        print("    please enter you service.  -s or --service")
    if my.username == 0 or my.password == 0 or my.service == 0:
        print("    more information please use -h or --help.")
        sys.exit(2)


def _write_header(buf, header_list):
    header_list.append(buf.decode('utf-8'))


def _discard(buf):
    pass


def get_url(url):
    curl = pycurl.Curl()
    headers = [
        "Accept:text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
        "User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0"
    ]
    header = []
    newurl = "http://" + url + "/"
    curl.setopt(pycurl.HTTPHEADER, headers)
    curl.setopt(pycurl.URL, newurl)
    curl.setopt(pycurl.HTTPGET, 1)
    curl.setopt(pycurl.HTTP_VERSION, pycurl.CURL_HTTP_VERSION_1_1)
    curl.setopt(pycurl.FOLLOWLOCATION, 5)
    curl.setopt(pycurl.HEADERFUNCTION, lambda buf: _write_header(buf, header))
    curl.setopt(pycurl.WRITEFUNCTION, _discard)
    curl.perform()
    curl.close()

    header_dict = {}
    for temp in header[1:]:
        if ":" in temp:
            key, value = temp.split(":", 1)
            header_dict[key.strip()] = value.strip()

    if "success" in header_dict["Location"]:
        print("already logged in")
        sys.exit(0)

    print("logging in...")
    _, queryString = header_dict["Location"].split("?", 1)
    return queryString


def transcode(queryString):
    encode_map = {
        '!': "%2521", '"': "%2522", '#': "%2523", '$': "%2524",
        '%': "%2525", '&': "%2526", '\'': "%2527", '(': "%2528",
        ')': "%2529", '*': "%252A", '+': "%252B", ',': "%252C",
        '/': "%252F", ':': "%253A", ';': "%253B", '<': "%253C",
        '=': "%253D", '>': "%253E", '?': "%253F", '@': "%2540",
        '[': "%255B", '\\': "%255C", ']': "%255D", '^': "%255E",
        '_': "%255F", '`': "%2560"
    }
    def replace(match):
        return encode_map.get(match.group(0), match.group(0))
    return re.sub(r'[!"#$%&\'()*+,/:;<=>?@\[\\\]^_`]', replace, queryString)


def login(url, queryString, my):
    curl = pycurl.Curl()
    headers = [
        "User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0",
        "Content-Type:application/x-www-form-urlencoded; charset=UTF-8",
        "Accept:*/*",
        "Accept-Encoding:gzip, deflate"
    ]
    body = []
    postdata = ("userId=" + my.username +
                "&password=" + my.password +
                "&service=" + my.service +
                "&queryString=" + queryString +
                "&operatorPwd=&operatorUserId=&validcode=&passwordEncrypt=false")
    curl.setopt(pycurl.HTTPHEADER, headers)
    curl.setopt(pycurl.URL, "http://" + url + "/eportal/InterFace.do?method=login")
    curl.setopt(pycurl.POSTFIELDS, postdata)
    curl.setopt(pycurl.HTTP_VERSION, pycurl.CURL_HTTP_VERSION_1_1)
    curl.setopt(pycurl.WRITEFUNCTION, lambda buf: _write_header(buf, body))
    curl.setopt(pycurl.HEADERFUNCTION, _discard)
    curl.perform()
    curl.close()

    data = json.loads(body[0])
    print("NOTICE:")
    print("    " + data['result'])
    print("    " + data['message'])
