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
all: getUrl

getUrl: main.o $(OBJS)
		$(cc) $(OPT) -o getUrl $(OBJS) main.o -L$(LIB)

clean:
	rm -f getUrl *~

cleanall:
	rm -f getUrl *~ *.o