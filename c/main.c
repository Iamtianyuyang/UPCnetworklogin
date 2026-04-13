/* UPC , 2024.5.4*/
#include<stdio.h>
#include<string.h>
#include<time.h>
#include<getopt.h>
#include<curl/curl.h>
#include<stdlib.h>

#define VERSION "1.0.0"
static const struct option longopts[] ={
    {"help", no_argument, NULL, 'h'},
    {"version", no_argument, NULL, 'V'},
    {"username", required_argument, NULL, 'u'},
    {"password", required_argument, NULL, 'p'},
    {"service", required_argument, NULL, 's'},
    {NULL, 0, NULL, 0}
};
struct response_buf
{
    int size;
    char *data;
};
struct user
{
    char *username;
    char *pwd;
    char *nettype;
};
struct CharDictionary
{
    char key;
    char value[6];
};
char *transcode(char *indata, int length)
{
    struct CharDictionary dict[] = {
        {'!', "%2521"},
        {'"', "%2522"},
        {'#', "%2523"},
        {'$', "%2524"},
        {'%', "%2525"},
        {'&', "%2526"},
        {'\'', "%2527"},
        {'(', "%2528"},
        {')', "%2529"},
        {'*', "%252A"},
        {'+', "%252B"},
        {',', "%252C"},
        {'/', "%252F"},
        {':', "%253A"},
        {';', "%253B"},
        {'<', "%253C"},
        {'=', "%253D"},
        {'>', "%253E"},
        {'?', "%253F"},
        {'@', "%2540"},
        {'[', "%255B"},
        {'\\', "%255C"},
        {']', "%255D"},
        {'^', "%255E"},
        {'_', "%255F"},
        {'`', "%2560"}
    };
    static unsigned int dictlength = 26;
    int i, j, k, sum;
    sum = 0;
    k = 0;
    for (i = 0; i < length; i++) {
        for (j = 0; j < dictlength; j++) {
            if (indata[i] == dict[j].key) {
                sum = sum + 1;
                break;
            }
        }
    }

    char *outdata;
    outdata = (char *)malloc((length + 10 + sum * 5) * sizeof(char));
    outdata = memset(outdata, '0' ,length + 10 + sum * 5);
    for (i = 0; i < length; i++) {
        outdata[k] = indata[i];
        for (j = 0; j < dictlength; j++) {
            if (indata[i] == dict[j].key) {
                strcpy(&outdata[k], dict[j].value);
                k = k + 4;
                break;
            }
        }
        k = k + 1;
    }
    return outdata;
}
static size_t write_callback(char *buffer, size_t size, size_t nitems, void *userdata)
{
    struct response_buf *buf = userdata;
    size_t realsize = nitems * size;
    char *temp = malloc(realsize + buf->size + 1);
    strcpy(&temp[0], buf->data);
    strcpy(&temp[buf->size], buffer);
    buf->data = temp;
    buf->size += realsize;
    return nitems * size;
}
static size_t discard_callback(char *buffer, size_t size, size_t nitems, void *userdata)
{
    return nitems * size;
}
char *jsessionid(int length)
{
    static const char charset[] = "0123456789QWERTYUIOPASDFGHJKLZXCVBNM";
    char *random_string = (char *)malloc((length+1)*sizeof(char));
    srand(time(NULL));
    for (int i = 0; i < length; i++){
        random_string[i] = charset[rand() % (sizeof(charset) - 1)];
    }
    random_string[length] = '\0';
    return random_string;
}
char *findresult(char* response, char flag[128])
{
    char *result;
    result = malloc(100 * sizeof(char));
    char *result_start = strstr(response, flag);
    if (result_start == NULL) {
        strcpy(result, "未找到字段");
        return result;
    }
    result_start += strlen(flag);
    char *result_end = strchr(result_start, '\"');
    if (result_end == NULL) {
        strcpy(result, "未找到结束引号");
        return result;
    }
    strncpy(result, result_start, result_end - result_start);
    result[result_end - result_start] = '\0';
    return result;
}
int getUrl(char* queryString)
{
    CURL *curl;
    CURLcode res;
    curl_global_init(CURL_GLOBAL_ALL);
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Accept:text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7");
    headers = curl_slist_append(headers, "User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0");
    curl  = curl_easy_init();
    if(curl)
    {
        struct response_buf headerdata = {0, ""};

        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_COOKIEFILE,"");
        curl_easy_setopt(curl, CURLOPT_COOKIEJAR,"");
        curl_easy_setopt(curl, CURLOPT_URL,"http://lan.upc.edu.cn/");
        curl_easy_setopt(curl, CURLOPT_HTTPGET,1L);
        curl_easy_setopt(curl, CURLOPT_HTTP_VERSION,CURL_HTTP_VERSION_1_1);
        curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION,5);
        curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, write_callback);
        curl_easy_setopt(curl, CURLOPT_HEADERDATA, &headerdata);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, discard_callback);

        res = curl_easy_perform(curl);
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
        if(res != CURLE_OK)
        {
            printf("连接失败\n");
            curl_global_cleanup();
            return 1;
        }
        char *line = strtok(headerdata.data, "\n");
        while(line != NULL) {
            if(strstr(line, "./success") != NULL) {
                printf("already logged in\n");
                curl_global_cleanup();
                exit(0);
            }
            if(strstr(line, "wlanuserip") != NULL) {
                strcpy(queryString, line + 50);
            }
            line = strtok(NULL, "\n");
        }
    }
    curl_global_cleanup();
    return 0;
}
int login(char *quertyString, char *username, char *password, char *service)
{
    CURL *curl;
    CURLcode res;
    curl_global_init(CURL_GLOBAL_ALL);
    struct curl_slist *headers = NULL;
    int length = 33;
    char *jsession = jsessionid(length);
    char *postdata = (char *)malloc(1000 * sizeof(char));
    headers = curl_slist_append(headers,"Host:lan.upc.edu.cn");
    headers = curl_slist_append(headers,"User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0");
    headers = curl_slist_append(headers,"Content-Type:application/x-www-form-urlencoded; charset=UTF-8");
    headers = curl_slist_append(headers,"Accept:*/*");
    headers = curl_slist_append(headers,"Origin:http://lan.upc.edu.cn");
    headers = curl_slist_append(headers,"Accept-Encoding:gzip, deflate");

    sprintf(postdata, "userId=%s&password=%s&service=%s&queryString=%s&operatorPwd=&operatorUserId=&validcode=&passwordEncrypt=false", username, password, service, quertyString);

    curl  = curl_easy_init();
    if(curl)
    {
        struct response_buf bodydata = {0, ""};
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_URL, "http://lan.upc.edu.cn/eportal/InterFace.do?method=login");
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, postdata);
        curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT_MS,2000);
        curl_easy_setopt(curl, CURLOPT_HTTP_VERSION, CURL_HTTP_VERSION_1_1);
        curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, discard_callback);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &bodydata);

        res = curl_easy_perform(curl);
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
        if(res == CURLE_OK)
        {
            printf("NOTICE:\n");
            printf("    %s\n", findresult(bodydata.data, "\"result\":\""));
            printf("    %s\n", findresult(bodydata.data, "\"message\":\""));
        }
        else
        {
            printf("登录失败\n");
        }
    }
    curl_global_cleanup();
    free(postdata);
    free(jsession);
    return 0;
}
static void print_help()
{
    printf("Option:\n");
    printf("    -h, --help          give you some information to better use this program\n");
    printf("    -V, --version       get this program version\n");
    printf("    -u, --username      your campus card ID\n");
    printf("    -p, --password      your campus card password\n");
    printf("    -s, --service       your campus network operator: cmcc or unicom or ctcc or default or local\n");
}
static void check_information(struct user my)
{
    if(my.username == NULL || my.pwd == NULL || my.nettype == NULL)
    {
        printf("WARNING:\n");
    }
    if(my.username == NULL)
    {
        printf("    please enter you username. -u or --username\n");
    }
    if(my.pwd == NULL)
    {
        printf("    please enter you password. -p or --password\n");
    }
    if(my.nettype == NULL)
    {
        printf("    please enter you service.  -s or --service\n");
    }
    if(my.username == NULL || my.pwd == NULL || my.nettype == NULL)
    {
        printf("    more information please use -h or --help.\n");
        exit(EXIT_SUCCESS);
    }
}
int main(int argc, char *argv[])
{
    char *queryString;
    struct user my;
    int optc;
    my.username = NULL;
    my.pwd = NULL;
    my.nettype = NULL;
    while((optc = getopt_long(argc, argv, "hVu:p:s:", longopts, NULL)) != -1)
    {
        switch (optc)
        {
        case 'h':
            print_help();
            exit(EXIT_SUCCESS);
            break;
        case 'V':
            printf("VERSION : %s\n",VERSION);
            exit(EXIT_SUCCESS);
            break;
        case 'u':
            my.username = optarg;
            break;
        case 'p':
            my.pwd = optarg;
            break;
        case 's':
            my.nettype = optarg;
            break;
        default:
            break;
        }
    }
    check_information(my);

    printf("username : %s\n", my.username);
    printf("service  : %s\n", my.nettype);
    queryString = (char *)malloc(200 * sizeof(char));
    getUrl(queryString);
    queryString = transcode(queryString, 200);
    login(queryString, my.username, my.pwd, my.nettype);
    free(queryString);
    return 0;
}
