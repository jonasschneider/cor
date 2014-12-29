all: date

clean:
	rm -fr date date.o dietlibc-0.33{,.tar.bz2}

CFLAGS=-nostdlib -nostdinc -Idietlibc-0.33/include -nodefaultlibs -Wall -Wextra -m64 -Werror -std=c11 -ggdb -c
CC=./sshwrap gcc
LD=./sshwrap ld

date.o: date.c dietlibc-0.33/include
	$(CC) $(CFLAGS) date.c -o date.o

date: date.o dietlibc-0.33/bin-x86_64
	$(LD) $< dietlibc-0.33/bin-x86_64/start.o -Ldietlibc-0.33/bin-x86_64 -ldietc -o $@

dietlibc-0.33/bin-x86_64: dietlibc-0.33
	./sshwrap 'cd dietlibc-0.33 && make && mv bin-x86_64/{dietlibc.a,libdietc.a}'

dietlibc-0.33/include: dietlibc-0.33
dietlibc-0.33:
	wget http://www.fefe.de/dietlibc/dietlibc-0.33.tar.bz2
	shasum -c dietlibc-0.33.tar.bz2.sha1
	tar xjf dietlibc-0.33.tar.bz2
