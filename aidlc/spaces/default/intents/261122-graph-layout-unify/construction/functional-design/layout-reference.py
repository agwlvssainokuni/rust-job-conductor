# 配置の参照実装(Python。設計の記録として残す。規範)。graph-layout-unify。
# `git log --graph` の出力の観察だけから自分で立てた規則に、「両方の飛び越し」(off-main の線 1 本)と分岐
# (off-main の線 2 本)を組み込んだ 1 つの仕組み。git のソースは読んでいない(git は GPL v2)。
# 主線の列 0 の `|` は、上のジョブが実行されうる(先頭のジョブからたどれる)、かつ次のジョブへの辺を持つときだけ描く。
# 検証: git の出力 537、1,517、4,757、4,348 例と全例一致。入力の skip: {i: ('n',t) | ('s',t) | ('b',t1,t2)}。
import re,sys
def parse(path):
    txt=open(path).read(); cases=[]
    for m in re.finditer(r"### jobs=(\d+) skips=([^\n]*)\n```text\n(.*?)\n```",txt,re.S):
        n=int(m.group(1)); sk={}
        if m.group(2).strip()!='-':
            for p in m.group(2).split():
                both='>>' in p
                a,b=p.split('>>') if both else p.split('>')
                tg=[n if x=='END' else ord(x)-97 for x in b.split(',')]
                if both: sk[ord(a)-97]=('s',tg[0])
                else: sk[ord(a)-97]=('n',tg[0]) if len(tg)==1 else ('b',tg[0],tg[1])
        cases.append((n,sk,m.group(3).split('\n')))
    return cases
def move_rows(c,d,m,R,nxt,c0):
    def fix(r): return (c0+r[1:]).rstrip()
    if c-d==1:
        return [fix('| '*(c-1)+'|/'+' /'*(m-c))], True
    us=c-d-2
    row1=''
    for j in range(c):
        row1+='|'
        if j<c-1: row1+= '_' if j>=(c-1)-us else ' '
    row1+='/'+' /'*(m-c)
    fd=len(R)
    for j in range(len(R)):
        if any(R[j]==R[jj] for jj in range(j)): fd=j;break
    row2='| '*d+'|/'+''.join(('/ ' if (j>=fd or (d>=1 and j>=len(R)-(m-c) and any(R[jj]==nxt for jj in range(len(R)-(m-c),j+1)))) else '| ') for j in range(d,len(R)))
    return [fix(row1),fix(row2)], False
def layout(n, skip):
    """skip: {i: ('n',t) | ('s',t) both-skip | ('b',t1,t2) branch}"""
    succ={}
    for i in range(n):
        sp=skip.get(i); e=[]
        if sp is None or sp[0]=='n': e.append(i+1)
        if sp: e+= list(sp[1:])
        succ[i]=e
    reach={0}; st=[0]
    while st:
        x=st.pop()
        for y in succ.get(x,[]):
            if y not in reach: reach.add(y); st.append(y)
    def mainok(j):
        if j<0 or j>=n: return False
        sp=skip.get(j)
        return (j in reach) and (sp is None or sp[0]=='n')
    out=[]; lanes=[]; after_fork=False; lastkind=None
    def settle(i,kk=None,tgt=None):
        nonlocal lanes,lastkind
        while True:
            pair=None
            for a in range(len(lanes)):
                for b in range(a+1,len(lanes)):
                    if lanes[a]==lanes[b] and lanes[a]!=tgt and not (kk is not None and a<kk<b): pair=(a,b);break
                if pair:break
            if not pair: return
            a,b=pair
            rest=[x for j,x in enumerate(lanes) if j!=b]
            rows,lm=move_rows(b+1,a+1,len(lanes),rest,i,'|' if mainok(i-1) else ' ')
            out.extend(rows); lastkind='merge1' if lm else 'dup2'
            lanes=rest
    for i in range(n+1):
        start=len(out); lastkind=None
        S=[j for j,x in enumerate(lanes) if x==i]
        is_target=bool(S)
        if is_target:
            settle(i,S[0],i)
            S=[j for j,x in enumerate(lanes) if x==i]; m=len(lanes); k=S[0]
            rem=[x for j,x in enumerate(lanes) if j!=k and x!=i]
            rows,lm=move_rows(k+1,0,m,rem,i,'|' if mainok(i-1) else ' ')
            out.extend(rows); lastkind='merge1' if lm else 'cross2'
            lanes=rem
        settle(i)
        if len(out)>start: after_fork=False
        existing=len(lanes)
        if i==n: out.append('o @end'); break
        sp=skip.get(i)
        kind=sp[0] if sp else None
        if kind in ('n','s'):
            t=sp[1]; join= bool(lanes) and lanes[0]==t
        else: join=False
        newcount=0 if (sp is None or join) else (2 if kind=='b' else 1)
        q=None;qonly=False
        if newcount==0 and (i+1) in lanes and lastkind=='merge1': q=lanes.index(i+1)
        elif sp is None and (i+1) in lanes and lastkind=='dup2' and lanes.index(i+1)==0: q=0;qonly=True
        eg='\\' if (newcount and after_fork) else '|'
        width=2*(1+existing+newcount)
        row='*'+''.join(' '+(('/' if (q is not None and ((j==q) if qonly else (j>=q))) else '|') if not newcount else eg) for j in range(existing))
        out.append(row.ljust(width)+chr(97+i))
        c0='|' if mainok(i) else ' '
        if newcount:
            out.append(c0+'\\'+' \\'*(existing+newcount-1))
            if kind=='b':
                lanes=[sp[1],sp[2]]+lanes
            else:
                lanes=[sp[1]]+lanes
        elif join:
            out.append(c0+'\\|'+' |'*(existing-1))
        after_fork=bool(newcount)
    return out
if __name__=='__main__':
    for f in sys.argv[1:]:
        c=parse(f); bad=0
        for n,sk,exp in c:
            got=[x.rstrip() for x in layout(n,sk)]
            if got!=exp:
                bad+=1
                if bad<=1: print(n,sk);print('\n'.join(exp));print('--');print('\n'.join(got))
        print(f,len(c),'bad',bad)
