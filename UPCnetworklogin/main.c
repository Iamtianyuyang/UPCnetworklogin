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
    {"sercive", required_argument, NULL, 's'},
    {NULL, 0, NULL, 0}
};
struct my_info
{
    int shoesize;
    char *secret;
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
            // printf("sum = %d\n",i);
            // printf("ddd %c org %c\n",dict[j].key,indata[i]);
            if (indata[i] == dict[j].key) {
                strcpy(&outdata[k], dict[j].value);
                // printf(" %s\n",dict[j].value);
                k = k + 4;
                break;  
            }
            
        }
        // printf("%c i %d k %d \n",indata[i], i, k);
        k = k + 1;
    }
    // printf("outdata %s\n",outdata);
    // printf("indata %s\n",indata);
    return outdata;
}
static size_t header_callback(char *buffer, size_t size, size_t nitems, void *userdata)
{
    struct my_info *i = userdata;
    size_t realsize = nitems * size;
    // i->secret = (char *)realloc(i->secret,i->shoesize + realsize + 1);
    char *temp = malloc(realsize + i->shoesize + 1);
    strcpy(&temp[0],i->secret);
    strcpy(&temp[i->shoesize],buffer);
    i->secret = temp;
    i->shoesize += realsize; 
    return nitems * size;
}
static size_t url_callback(char *buffer, size_t size, size_t nitems, void *userdata)
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
        return 0;
    } 
    result_start += strlen(flag);
    char *result_end = strchr(result_start, '\"');
    if (result_end == NULL) {
        strcpy(result, "未找到结束引号");
        return 0;
    }
    strncpy(result, result_start, result_end - result_start);
    result[result_end - result_start] = '\0'; 
    return result;

}
int getUrl(char *filename , char* queryString)
{   
    CURL *curl;
    CURLcode res;
    char *loginURl;
    struct curl_slist *list = NULL;
    curl_global_init(CURL_GLOBAL_ALL);
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Accept:text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7");
    headers = curl_slist_append(headers, "User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0");
    curl  = curl_easy_init();
    if(curl)
    {  
       struct my_info headerdata = {0 , ""};
       
       curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
       curl_easy_setopt(curl, CURLOPT_COOKIEFILE,"");
       curl_easy_setopt(curl, CURLOPT_COOKIEJAR,"");
       curl_easy_setopt(curl, CURLOPT_URL,"http://lan.upc.edu.cn/");
       curl_easy_setopt(curl, CURLOPT_HTTPGET,1L);
       curl_easy_setopt(curl, CURLOPT_HTTP_VERSION,CURL_HTTP_VERSION_1_1);
       curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION,5);
       curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION,header_callback);
       curl_easy_setopt(curl, CURLOPT_HEADERDATA, &headerdata);
       curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, url_callback);


    //    curl_easy_setopt(curl, CURLOPT_HEADERDATA, fp);
       res = curl_easy_perform(curl);
       if(res == CURLE_OK)
       {
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
       } 
       if(res != CURLE_OK)
       {
        printf("失败");
       }
       char *line = strtok(headerdata.secret,"\n");
       int lineNumber = 1;
       while(line != NULL){
        if(strstr(line, "wlanuserip") != NULL){
            char temp[strlen(line)-50];
            memcpy(temp,line + 50,strlen(line)-49);
            loginURl = malloc(strlen(line)-50);
            loginURl = temp;
            strcpy(queryString,loginURl);
        }
	if(strstr(line, "./success") != NULL)
        {
		printf("success\n");
                exit(0);
        }

        line = strtok(NULL, "\n");
        lineNumber++;   
    }
    }
    curl_global_cleanup();
    curl_slist_free_all(list);
    return 0;
}
int login(char *quertyString, char *username, char *password, char *service)
{
    CURL *curl;
    CURLcode res;
    curl_global_init(CURL_GLOBAL_ALL);
    struct curl_slist *headers = NULL;
    int length = 33;
    char *jsession = (char *)malloc((length + 1) * sizeof(char));
    char *cookie = (char *)malloc(200 * sizeof(char));
    char *postdata = (char *)malloc(1000 * sizeof(char));
    jsession = jsessionid(length);
    // printf("%s\n",jsession);
    headers = curl_slist_append(headers,"Host:lan.upc.edu.cn");
    // headers = curl_slist_append(headers,"Content-Length:344");
    // headers = curl_slist_append(headers,"Connection:keep-alive");
    headers = curl_slist_append(headers,"User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0");
    headers = curl_slist_append(headers,"Content-Type:application/x-www-form-urlencoded; charset=UTF-8");
    headers = curl_slist_append(headers,"Accept:*/*");
    headers = curl_slist_append(headers,"Origin:http://lan.upc.edu.cn");
    headers = curl_slist_append(headers,"Accept-Encoding:gzip, deflate");
    
    sprintf(cookie, "Cookie:EPORTAL_COOKIE_PASSWORD=; EPORTAL_COOKIE_USERNAME=; EPORTAL_COOKIE_DOMAIN=true; EPORTAL_COOKIE_OPERATORPWD=; JSESSIONID=%s", jsession);
    // printf("%s\n",cookie);
    // headers = curl_slist_append(headers,cookie);
    sprintf(postdata, "userId=%s&password=%s&service=%s&queryString=%s&operatorPwd=&operatorUserId=&validcode=&passwordEncrypt=false", username, password, service, quertyString);

    curl  = curl_easy_init();
    if(curl)
    {
        struct my_info headerdata = {0 , ""};
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_URL, "http://lan.upc.edu.cn/eportal/InterFace.do?method=login");
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, postdata);
        curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT_MS,2000);
        curl_easy_setopt(curl, CURLOPT_HTTP_VERSION, CURL_HTTP_VERSION_1_1);
        curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, url_callback);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, header_callback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &headerdata);
        
        res = curl_easy_perform(curl);
        if(res == CURLE_OK)
        {
         curl_slist_free_all(headers);
         curl_easy_cleanup(curl);
         printf("NOTICE:\n");
         printf("    %s\n",findresult(headerdata.secret, "\"result\":\""));
         printf("    %s\n",findresult(headerdata.secret, "\"message\":\""));
        } 
        if(res != CURLE_OK)
        {
         printf("failed!\n");
        }
    }
    curl_global_cleanup();
    free(postdata);
    free(cookie);
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
        printf("    please enter you sercive.  -s or --service\n");
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

    printf("username : %s\n",my.username);
    printf("service : %s\n",my.nettype);
    queryString = (char *)malloc(200 * sizeof(char));
    getUrl("./get2.html",queryString);
    queryString = transcode(queryString , 200);
    login(queryString, my.username, my.pwd, my.nettype);
    free(queryString);
    return 0;
}
