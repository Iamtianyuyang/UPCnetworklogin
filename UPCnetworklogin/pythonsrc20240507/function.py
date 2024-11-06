import re
import sys
import json
import pycurl
from io import BytesIO

def check_full_info(my):
    if my.username == 0 or my.password == 0 or my.service == 0 :
        print("WARNING:\n")
    if my.username == 0 :
        print("    please enter you username. -u or --username\n")
    if my.password == 0 :
        print("    please enter you password. -p or --password\n")
    if my.service == 0 :
        print("    please enter you sercive.  -s or --service\n")
    if my.username == 0 and my.password == 0 and my.service == 0 :
        print("    more information please use -h or --help.\n")
        sys.exit(2)
    if my.username == 0 or my.password == 0 or my.service == 0 :
        sys.exit(2)



def write_header(buf, header_data):
    header_data.append(buf.decode('utf-8'))



def write_body(buf):
    buf = buf 


   
def get_url(url):
    buffer = BytesIO()
    curl = pycurl.Curl()
    headers = ["Accept:text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
               "User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0"]
    header = [];
    newurl = "http://" + url +"/" 
    curl.setopt(pycurl.HTTPHEADER, headers)
    curl.setopt(pycurl.URL, newurl)
    curl.setopt(pycurl.HTTPGET, 1)
    curl.setopt(pycurl.HTTP_VERSION, pycurl.CURL_HTTP_VERSION_1_1)
    curl.setopt(pycurl.FOLLOWLOCATION, 5)
    curl.setopt(pycurl.WRITEHEADER, buffer)
    curl.setopt(pycurl.HEADERFUNCTION, lambda buf: write_header(buf, header))
    curl.setopt(pycurl.WRITEFUNCTION, write_body)
    curl.perform()
    curl.close()
    # print(header)
    header_dict = {}
    for temp in header[1:]:
        if ":" in temp:
            key, value = temp.split(":", 1)
            header_dict[key.strip()] = value.strip()
    # print(header_dict["Location"])
    if "success" in header_dict["Location"]:
        print("logined")
        sys.exit(2)
    else:
        print("logining")
        temp, queryString = header_dict["Location"].split("?",1)
        # print(queryString)
    return queryString


def transcode(queryString):
    dict = {
    '!' : "%2521",
    '"' : "%2522",
    '#' : "%2523",
    '$' : "%2524",
    '%' : "%2525",
    '&' : "%2526",
    '\'' : "%2527",
    '(' : "%2528",
    ')' : "%2529",
    '*' : "%252A",
    '+' : "%252B",
    ',' : "%252C",
    '/' : "%252F",
    ':' : "%253A",
    ';' : "%253B",
    '<' : "%253C",
    '=' : "%253D",
    '>' : "%253E",
    '?' : "%253F",
    '@' : "%2540",
    '[' : "%255B",
    '\\' : "%255C",
    ']' : "%255D",
    '^' : "%255E",
    '_' : "%255F",
    '`' : "%2560"}
    def replace_key_with_value(match):
        key = match.group(0)
        return dict.get(key, key)
    newqueryString = re.sub(r'[!\"#$%&\'()*+,-/:;<=>?@\[\\\]^_`]', replace_key_with_value, queryString)
    # print(newqueryString)
    return newqueryString


def login(url, queryString, my):
    buffer = BytesIO()
    curl = pycurl.Curl()
    headers = ["Accept:text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
               "User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0",
               "Content-Type:application/x-www-form-urlencoded; charset=UTF-8",
               "Accept:*/*",
               "Accept-Encoding:gzip, deflate"]
    body = []
    postdata = "userId="+my.username+"&password="+my.password+"&service="+my.service+"&queryString="+queryString+"&operatorPwd=&operatorUserId=&validcode=&passwordEncrypt=false"
    curl.setopt(pycurl.HTTPHEADER, headers)
    curl.setopt(pycurl.URL, "http://"+url+"/eportal/InterFace.do?method=login")
    curl.setopt(pycurl.POSTFIELDS, postdata)
    curl.setopt(pycurl.HTTP_VERSION, pycurl.CURL_HTTP_VERSION_1_1)
    curl.setopt(pycurl.FOLLOWLOCATION, 5)
    curl.setopt(pycurl.WRITEDATA, buffer)
    curl.setopt(pycurl.WRITEFUNCTION, lambda buf: write_header(buf, body))
    curl.setopt(pycurl.HEADERFUNCTION, write_body)
    curl.perform()
    curl.close()
    data = json.loads(body[0])
    print("NOTICE:")
    print(" "+data['result'])
    print(" "+data['message'])