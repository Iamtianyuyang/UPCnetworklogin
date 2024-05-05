INC = ./
LIB = ./
LIB += -lcurl
OPT = -O3 -std=c99 -Wall 
OBJS = 

cc = gcc

.C.o:
	$(CC) $(OPT) -I$(INC) -c $<
.c.o:
	$(CC) $(OPT) -I$(INC) -c $<
.f.o:
	$(FC) $(FOPT) -c $<
.F.o:
	$(FC) $(FOPT) -c $<
all: login

login: main.o $(OBJS)
		$(cc) $(OPT) -o login $(OBJS) main.o -L$(LIB)

clean:
	rm -f login *~

cleanall:
	rm -f logim *~ *.o