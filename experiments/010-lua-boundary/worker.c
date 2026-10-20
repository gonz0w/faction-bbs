/* Disposable Lua boundary worker. Not an OS sandbox or production door host. */
#include <lua.h>
#include <lauxlib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <unistd.h>
typedef struct { size_t live, peak; unsigned ticks, events; } Budget;
#define HEAP_LIMIT (1024u * 1024u)
static void *limited_alloc(void *ud, void *ptr, size_t old, size_t size) {
    Budget *b=ud; if (!ptr) old=0;
    if (!size) { free(ptr); b->live-=old; return NULL; }
    if (size > HEAP_LIMIT - (b->live-old)) return NULL;
    void *next=realloc(ptr,size); if (!next) return NULL;
    b->live=b->live-old+size; if(b->live>b->peak)b->peak=b->live;
    return next;
}
static Budget *budget(lua_State *L) { void *ud; lua_getallocf(L,&ud); return ud; }
static void tick(lua_State *L, lua_Debug *ar) {
    (void)ar; if(++budget(L)->ticks>20)luaL_error(L,"instruction budget");
}
static int emit(lua_State *L) {
    size_t len; const char *s=luaL_checklstring(L,1,&len);
    if(len>120)return luaL_error(L,"output length");
    for(size_t i=0;i<len;i++)if((unsigned char)s[i]<32 || (unsigned char)s[i]>126)
        return luaL_error(L,"output controls");
    if(++budget(L)->events>8)return luaL_error(L,"event budget");
    const char *verb=lua_tostring(L,lua_upvalueindex(1));
    printf("%s|%.*s\n",verb,(int)len,s); fflush(stdout); return 0;
}
static void api(lua_State *L,const char *name,const char *verb) {
    lua_pushstring(L,verb);lua_pushcclosure(L,emit,1);lua_setglobal(L,name);
}
int main(int argc,char **argv) {
    struct rlimit core={0,0}; if(setrlimit(RLIMIT_CORE,&core))return 7;
    if(argc!=2)return 2;
    if(!strcmp(argv[1],"--crash"))abort();
    if(!strcmp(argv[1],"--stall"))for(;;)pause();
    FILE *f=fopen(argv[1],"rb");if(!f)return 2;
    char code[4097];size_t len=fread(code,1,sizeof code,f);int bad=ferror(f);fclose(f);
    if(bad || len>4096)return 2;
    Budget b={0};
#if LUA_VERSION_NUM >= 505
    lua_State *L=lua_newstate(limited_alloc,&b,0);
#else
    lua_State *L=lua_newstate(limited_alloc,&b);
#endif
    if(!L)return 3;
    /* No standard libraries, loaders, io/os/package/debug, pcall or coroutines.
       The only host functions emit proposals; Rust alone grants domain access. */
    api(L,"menu","MENU");api(L,"room_post","ROOM_POST");
    lua_sethook(L,tick,LUA_MASKCOUNT,1000);
    int status=luaL_loadbufferx(L,code,len,"fixture","t");
    if(!status)status=lua_pcall(L,0,0,0);
    /* Error classes only: never print script source, arguments or private content. */
    const char *reason="ok";
    if(status==LUA_ERRMEM)reason="memory";
    else if(status){const char *e=lua_tostring(L,-1);
        if(e && strstr(e,"instruction budget"))reason="instruction";
        else if(e && strstr(e,"output length"))reason="length";
        else if(e && strstr(e,"output controls"))reason="controls";
        else if(e && strstr(e,"event budget"))reason="events";
        else reason="script";
    }
    lua_close(L);
    fprintf(stderr,"result=%s peak_lua_bytes=%zu live_after_close=%zu\n",reason,b.peak,b.live);
    return status?4:0;
}
