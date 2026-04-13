import getopt
import sys

VERSION = '1.0.0'

HELP_TEXT = """Option:
    -h, --help          give you some information to better use this program
    -V, --version       get this program version
    -u, --username      your campus card ID
    -p, --password      your campus card password
    -s, --service       your campus network operator: cmcc or unicom or ctcc or default or local"""


def get_opt(my):
    short_opts = 'u:p:s:hV'
    long_opts = ['username=', 'password=', 'service=', 'help', 'version']
    try:
        args, _ = getopt.getopt(sys.argv[1:], short_opts, long_opts)
    except getopt.GetoptError:
        print('参数错误\n' + HELP_TEXT)
        sys.exit(2)

    for opt, arg in args:
        if opt in ('-u', '--username'):
            my.username = arg
        elif opt in ('-p', '--password'):
            my.password = arg
        elif opt in ('-s', '--service'):
            my.service = arg
        elif opt in ('-h', '--help'):
            print(HELP_TEXT)
            sys.exit(0)
        elif opt in ('-V', '--version'):
            print("Version: " + VERSION)
            sys.exit(0)
    return my
