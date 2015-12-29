# Makefile for ash.
#
# Copyright (C) 1989 by Kenneth Almquist.  All rights reserved.
# This file is part of ash, which is distributed under the terms specified
# by the Ash General Public License.  See the file named LICENSE.

FILES=main.o options.o parser.o eval.o expand.o jobs.o redir.o exec.o\
	builtins.o cd.o miscbltin.o mail.o var.o input.o output.o nodes.o syntax.o\
	signames.o memalloc.o error.o trap.o show.o dirent.o mystring.o\
	init.o

CFILES=main.c options.c parser.c eval.c expand.c jobs.c redir.c exec.c\
	builtins.c cd.c miscbltin.c mail.c var.c input.c output.c nodes.c syntax.c\
	signames.c memalloc.c error.c trap.c show.c dirent.c mystring.c

GENERATEDFILES=syntax.h syntax.c signames.h signames.c nodes.c nodes.c\
	builtin.h builtin.c init.c token.def

#MALLOC=mymalloc.o


#CC=gcc
DEBUG=-g
CFLAGS=$(DEBUG)
LDFLAGS=
BLTIN=bltin


all: make_bltin ash

make_bltin:
	cd bltin; make 'CC=$(CC)' 'DEBUG=$(DEBUG)'


clean:
	rm -f $(FILES)
	rm -f $(GENERATEDFILES) mksyntax mksignames mknodes mkinit
	rm -f bltin/bltinlib.a bltin/*.o bltin/operators.h bltin/operators.c

clobber: clean
	rm -f ash bltin/catf bltin/expr bltin/test 'bltin/[' bltin/echo bltin/line bltin/nlecho bltin/true bltin/: bltin/umask


ash:$P $(FILES) $(BLTIN)/bltinlib.a $(MALLOC)
	$(CC) -o temp $(LDFLAGS) $(DEBUG) $(FILES) $(BLTIN)/bltinlib.a $(MALLOC)
#        ld -o temp crt0.o $(FILES) $(BLTIN)/bltinlib.a $(MALLOC) -lc
	mv -f temp $@

lint:
	lint $(CFILES) init.c

syntax.c syntax.h: mksyntax
	./mksyntax

mksyntax: mksyntax.c parser.h
	$(CC) -o mksyntax mksyntax.c

signames.c signames.h: mksignames
	./mksignames

mksignames: mksignames.c
	$(CC) -o mksignames mksignames.c

nodes.c nodes.h: mknodes nodetypes nodes.c.pat
	./mknodes

mknodes: mknodes.c
	$(CC) -o mknodes -g mknodes.c

token.def: mktokens
	sh mktokens

builtins.h builtins.c: mkbuiltins builtins
	sh mkbuiltins
	rm -f builtins.o

.c:
	echo make is confused, it but should recover

init.o: mkinit $(CFILES)
	./mkinit '$(CC) -c $(CFLAGS) init.c' $(CFILES)

mkinit: mkinit.c
	$(CC) -o mkinit mkinit.c


cd.o: shell.h var.h nodes.h jobs.h options.h output.h memalloc.h error.h\
	mystring.h
dirent.o: shell.h mydirent.h
eval.o: shell.h nodes.h syntax.h expand.h parser.h jobs.h eval.h builtins.h\
	options.h exec.h redir.h input.h output.h trap.h var.h memalloc.h\
	error.h mystring.h
error.o: shell.h main.h options.h output.h error.h
exec.o: shell.h main.h nodes.h parser.h redir.h eval.h exec.h builtins.h var.h\
	options.h input.h output.h memalloc.h error.h init.h\
	mystring.h
expand.o: shell.h main.h nodes.h eval.h expand.h syntax.h parser.h jobs.h\
	options.h var.h input.h output.h memalloc.h error.h\
	mystring.h mydirent.h
input.c: shell.h syntax.h input.h output.h memalloc.h error.h
jobs.o: shell.h main.h parser.h nodes.h jobs.h options.h trap.h signames.h\
	syntax.h input.h output.h memalloc.h error.h mystring.h
mail.o: shell.h exec.h var.h output.h memalloc.h error.h
main.o: shell.h mail.h options.h var.h output.h parser.h nodes.h eval.h jobs.h\
	input.h trap.h error.h memalloc.h init.h
memalloc.o: shell.h output.h memalloc.h error.h machdep.h mystring.h
miscbltin.o: shell.h options.h var.h output.h memalloc.h error.h mystring.h
mystring.o: shell.h syntax.h error.h mystring.h
nodes.o: shell.h nodes.h memalloc.h machdep.h mystring.h
options.o: shell.h options.h nodes.h eval.h jobs.h input.h output.h trap.h\
	var.h memalloc.h error.h mystring.h
output.o: shell.h syntax.h output.h memalloc.h error.h
parser.o: shell.h parser.h nodes.h expand.h redir.h syntax.h options.h input.h\
	output.h var.h error.h memalloc.h mystring.h token.def
redir.o: shell.h nodes.h jobs.h expand.h redir.h output.h memalloc.h error.h
show.o: shell.h parser.h nodes.h mystring.h
syntax.o: shell.h syntax.h
trap.o: shell.h main.h nodes.h eval.h jobs.h options.h syntax.h signames.h\
	output.h memalloc.h error.h trap.h
var.o: shell.h output.h expand.h nodes.h eval.h exec.h syntax.h mail.h\
	options.h var.h memalloc.h error.h mystring.h
