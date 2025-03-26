import getopt
import sys
VERSION='1.0.0'

def get_opt(my):
    short_opts = 'u:p:s:hV'
    long_opts = ['username=', 'password=', 'service=', 'help=', 'Version=']
    try:
        args, remainder = getopt.getopt(sys.argv[1:], short_opts, long_opts)
    except getopt.GetoptError:
        print('参数错误')
        print("    -h, --help          give you some information to better use this program\n")
        print("    -V, --version       get this program version\n")
        print("    -u, --username      your campus card ID\n")
        print("    -p, --password      your campus card password\n")
        print("    -s, --service       your campus network operator: cmcc or unicom or ctcc or default or local\n")
        sys.exit(2)
    
    for opt, arg in args:
        if opt in ('-u', '--username'):
            my.username = arg
        elif opt in ('-p', '--password'):
            my.password = arg
        elif opt in ('-s', '--service'):
            my.service = arg
        elif opt in ('-h', '--help'):
            print("Option:\n")
            print("    -h, --help          give you some information to better use this program\n")
            print("    -V, --version       get this program version\n")
            print("    -u, --username      your campus card ID\n")
            print("    -p, --password      your campus card password\n")
            print("    -s, --service       your campus network operator: cmcc or unicom or ctcc or default or local\n")
            sys.exit(2)
        elif opt in ('-V', '--Version'):
            print("Version:"+VERSION)
            sys.exit(2)
    return my
    