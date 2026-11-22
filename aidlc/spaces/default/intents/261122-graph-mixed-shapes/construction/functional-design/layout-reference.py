import re,sys
# 配置の参照実装(Python。リポジトリの外の試作の写し。設計の記録として残す)。
# `git log --graph` の出力の観察だけから、自分で立てた規則。git のソースは読んでいない(git は GPL v2)。
# 検証: 交差と同じ行き先が混ざる 4,348 例(ジョブ 4〜7 個)、既存の 537、1,517、4,757 例、ジョブ 8〜11 個の
# ランダム 1,517 例と、1 文字も違わず一致した。入力: `layout(n, skip)`。n はジョブの数、skip は
# {ジョブの位置: 行き先の位置(終点は n)}(通常の飛び越しだけのグラフ)。出力は行の並び。
def parse(path):
    import re
    txt=open(path).read(); cases=[]
    for m in re.finditer(r"### jobs=(\d+) skips=([^\n]*)\n```text\n(.*?)\n```",txt,re.S):
        n=int(m.group(1)); sk={}
        if m.group(2).strip()!='-':
            for p in m.group(2).split():
                a,b=p.split('>'); sk[ord(a)-97]= n if b=='END' else ord(b)-97
        cases.append((n,sk,m.group(3).split('\n')))
    return cases
def move_rows(c,d,m,R=None,nxt=None):
    """lane at column c (1-based) merges into column d (0=main) ; m lanes total (cols 1..m).
    returns (rows, lastmerge flag)"""
    if c-d==1:
        return [('| '*(c-1)+'|/'+' /'*(m-c)).rstrip()], True
    gaps=c-1
    us=c-d-2
    s=''
    for g in range(gaps):
        s+='|'
        s+= '_' if g>=gaps-us else ' '
    s+='|'  # col c-1 at pos 2(c-1)
    # position: cols 0..c-1 => c lanes; we built gaps+1 '|' with gap chars between; then '/' for moving lane
    row1=''
    for j in range(c):
        row1+='|'
        if j<c-1: row1+= '_' if j>=(c-1)-us else ' '
    row1+='/'
    row1+=' /'*(m-c)
    if R is None: R=[object()]*(m-1)
    fd=len(R)
    for j in range(len(R)):
        if any(R[j]==R[jj] for jj in range(j)): fd=j;break
    row2='| '*d+'|/'+''.join(('/ ' if (j>=fd or (d>=1 and j>=len(R)-(m-c) and any(R[jj]==nxt for jj in range(len(R)-(m-c),j+1)))) else '| ') for j in range(d,len(R)))
    return [row1.rstrip(),row2.rstrip()], False
def layout(n, skip):
    CUR=[0]
    out=[]; lanes=[]; after_fork=False; lastmerge=False; lastkind=None
    def settle(rem,kk=None,tgt=None):
        nonlocal lastmerge,lastkind
        while True:
            pair=None
            for a in range(len(rem)):
                for b in range(a+1,len(rem)):
                    if rem[a]==rem[b] and rem[a]!=tgt and not (kk is not None and a<kk<b): pair=(a,b);break
                if pair:break
            if not pair: break
            a,b=pair
            rest=[x for j,x in enumerate(rem) if j!=b]
            rows,lm=move_rows(b+1,a+1,len(rem),rest,CUR[0])
            out.extend(rows); lastmerge=lm; lastkind='merge1' if lm else 'dup2'
            rem=rest
        return rem
    for i in range(n+1):
        start_len=len(out); lastkind=None; CUR[0]=i
        S=[j for j,x in enumerate(lanes) if x==i]
        is_target=bool(S)
        if is_target:
            m=len(lanes); k=S[0]
            lanes=settle(lanes,k,i)
            S=[j for j,x in enumerate(lanes) if x==i]; m=len(lanes); k=S[0]
            rem=[x for j,x in enumerate(lanes) if j!=k and x!=i]
            rows,lm=move_rows(k+1,0,m,rem)
            if not lm and len(rem)!=m-1: pass
            out.extend(rows); lastmerge=lm; lastkind='merge1' if lm else 'cross2'
            lanes=rem
            after_fork=False
        else: lastmerge=False
        lanes=settle(lanes)
        if len(out)>start_len: after_fork=False
        existing=len(lanes)
        if i==n: out.append('o @end'); break
        t=skip.get(i)
        join = t is not None and lanes and lanes[0]==t
        newlane = t is not None and not join
        q=None; qonly=False
        if (not newlane) and (i+1) in lanes and lastkind=='merge1': q=lanes.index(i+1)
        elif t is None and (i+1) in lanes and lastkind=='dup2' and lanes.index(i+1)==0: q=0; qonly=True
        eg='\\' if (newlane and after_fork) else '|'
        width=2*(1+existing+(1 if newlane else 0))
        row='*'+''.join(' '+(('/' if (q is not None and ((j==q) if qonly else (j>=q))) else '|') if not newlane else eg) for j in range(existing))
        out.append(row.ljust(width)+chr(97+i))
        if newlane:
            out.append('|\\'+' \\'*existing)
            lanes.insert(0,t)
        elif join: out.append(('|\\|'+' |'*(existing-1)))
        after_fork = newlane
    return out
if __name__=='__main__':
    c=parse(sys.argv[1] if len(sys.argv)>1 else 'layout-oracle-mixed.md'); bad=0; shown=0
    lim=int(sys.argv[2]) if len(sys.argv)>2 else 0
    for n,sk,exp in c:
        got=[x.rstrip() for x in layout(n,sk)]
        if got!=exp:
            bad+=1
            if shown<lim:
                shown+=1; print(n,sk);print('\n'.join(exp));print('--');print('\n'.join(got));print()
    print('total',len(c),'bad',bad)
