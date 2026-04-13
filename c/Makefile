CC = gcc
CFLAGS = -O2 -std=c99 -Wall
LDFLAGS = -lcurl
TARGET = login

all: $(TARGET)

$(TARGET): main.o
	$(CC) $(CFLAGS) -o $(TARGET) main.o $(LDFLAGS)

main.o: main.c
	$(CC) $(CFLAGS) -c main.c

clean:
	rm -f $(TARGET) main.o *~

.PHONY: all clean
