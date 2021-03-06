#define BSIZE 1024

/*
 * Copyright (C) 1989 by Kenneth Almquist.  All rights reserved.
 * This file is part of ash, which is distributed under the terms specified
 * by the Ash General Public License.  See the file named LICENSE.
 */

/*
 * The follow should be set to reflect the type of system you have:
 *        JOBS -> 1 if you have Berkeley job control, 0 otherwise.
 *        SYMLINKS -> 1 if your system includes symbolic links, 0 otherwise.
 *        DIRENT -> 1 if your system has the SVR3 directory(3X) routines.
 *        UDIR -> 1 if you want the shell to simulate the /u directory.
 *        ATTY -> 1 to include code for atty(1).
 *        SHORTNAMES -> 1 if your linker cannot handle long names.
 *        define BSD if you are running 4.2 BSD or later.
 *        define SYSV if you are running under System V.
 *        define DEBUG to turn on debugging.
 *
 * When debugging is on, debugging info will be written to $HOME/trace and
 * a quit signal will generate a core dump.
 */


#define JOBS 1
#define SYMLINKS 1
#define DIRENT 0
#define UDIR 1
#define ATTY 1
#define SHORTNAMES 0
#define BSD
/* #define SYSV */
/* #define DEBUG */

#if SHORTNAMES
#include "shortnames.h"
#endif


#ifdef __STDC__
typedef void *pointer;
#ifndef NULL
#define NULL (void *)0
#endif
#else /* not __STDC__ */
#define const
#define volatile
typedef char *pointer;
#ifndef NULL
#define NULL 0
#endif
#endif /* __STDC__ */
#define STATIC        /* empty */
#define MKINIT        /* empty */

extern char nullstr[1];                /* null string */


#ifdef DEBUG
#define TRACE(param)        trace param
#else
#define TRACE(param)
#endif
