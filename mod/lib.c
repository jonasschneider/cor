// http://linux.die.net/man/3/memset
__assert_fail
errno
fwrite

void cor_hitmarker() {
  while(1);
}

void memset(void *s, int c, int n) {
  __builtin_memset(s,c,n);
}

memcmp
memcpy
memmove


abort
fflush
fprintf
fputs

__get_cpu_features
_GLOBAL_OFFSET_TABLE_

_start
stderr
_Unwind_GetIP
_Unwind_GetLanguageSpecificData
_Unwind_GetRegionStart
_Unwind_SetGR
_Unwind_SetIP
