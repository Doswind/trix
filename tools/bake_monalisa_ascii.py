import os, sys
from PIL import Image, ImageFilter, ImageDraw, ImageFont
import numpy as np
ROOT=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REF=os.path.join(ROOT, 'docs/proposals/monalisa-v2/ref.webp')
im=Image.open(REF).convert('RGB')
FS=20; FONT='/System/Library/Fonts/Menlo.ttc'
f=ImageFont.truetype(FONT,FS); cw=max(1,round(f.getlength('M'))); a,d=f.getmetrics(); chh=a+d
box=(200,110,1140,1800); crop=im.crop(box); W,H=crop.size
poly=[(600,158),(640,145),(690,158),(735,195),(775,255),(805,330),(825,420),(832,510),
      (820,590),(790,665),(755,715),(850,735),(950,770),(1030,830),(1080,910),(1105,1010),
      (1110,1120),(1090,1260),(1060,1400),(1035,1530),(1000,1635),(950,1700),(900,1745),
      (840,1748),(780,1700),(710,1645),(640,1610),(560,1590),(485,1560),(420,1530),
      (370,1480),(330,1400),(305,1300),(290,1180),(280,1060),(285,940),(305,830),
      (340,740),(380,665),(400,580),(402,490),(415,400),(440,315),(470,240),(515,190),(560,165)]
mask=Image.new('L',(W,H),0); ImageDraw.Draw(mask).polygon([(x-box[0],y-box[1]) for x,y in poly],fill=255)
arr=np.asarray(crop).astype(np.float32); mx=arr.max(2); mn=arr.min(2)
sat=(mx-mn)/np.maximum(mx,1); lum=(0.2126*arr[:,:,0]+0.7152*arr[:,:,1]+0.0722*arr[:,:,2])/255
m=np.asarray(mask).copy(); m[(lum>0.5)&(sat<0.16)]=0
mask=Image.fromarray(m).filter(ImageFilter.MaxFilter(7)).filter(ImageFilter.MinFilter(7)).filter(ImageFilter.GaussianBlur(3))
clean=crop.filter(ImageFilter.MedianFilter(5)).filter(ImageFilter.GaussianBlur(2.5))
s=np.asarray(clean).astype(np.float32)/255; s=np.clip(s*1.10+0.01,0,1)
clean=Image.fromarray((s*255).astype(np.uint8))
cols,rows=77,56
col=clean.resize((cols,rows),Image.LANCZOS); al=mask.resize((cols,rows),Image.LANCZOS)
rgb=np.asarray(col).astype(np.float32); aa=np.asarray(al).astype(np.float32)/255
L=(0.2126*rgb[:,:,0]+0.7152*rgb[:,:,1]+0.0722*rgb[:,:,2])/255
g=[[' ']*cols for _ in range(rows)]
def put(x,y,ch,force=True):
    if 0<=x<cols and 0<=y<rows and (force or g[y][x]!=' '): g[y][x]=ch
def line(x0,y0,x1,y1,ch):
    dx=abs(x1-x0); sx=1 if x0<x1 else -1; dy=-abs(y1-y0); sy=1 if y0<y1 else -1; err=dx+dy
    while True:
        put(x0,y0,ch)
        if x0==x1 and y0==y1: break
        e2=2*err
        if e2>=dy: err+=dy; x0+=sx
        if e2<=dx: err+=dx; y0+=sy

# base tone fill inside matte
for y in range(rows):
    for x in range(cols):
        if aa[y,x]<0.3: continue
        # hair region
        hair = y<27 and abs(x-35)<20 and not (((x-35)/12.0)**2+((y-11.5)/8.0)**2<=1.0)
        if hair:
            ch=':' if L[y,x]<0.55 else '%'
            if L[y,x]<0.20: ch='.'
        elif y>=20:
            # dress vs chest/hands: bright skin stays bright
            if L[y,x]>0.55: ch='#' if L[y,x]<0.72 else '@'
            elif L[y,x]>0.38: ch='%'
            elif L[y,x]>0.22: ch='='
            else: ch=':' if L[y,x]>0.10 else '.'
        else:
            ch='%' if L[y,x]<0.5 else '#'
        g[y][x]=ch
# hair edge highlight and strands
for y in range(2,26):
    if y<5: hw=5+(y-1)*3.0
    elif y<14: hw=17
    elif y<22: hw=19
    else: hw=17-(y-22)*1.2
    put(int(35-hw),y,'%'); put(int(35+hw),y,'%')
    if y%3==0:
        put(int(35-hw+2),y,'\\'); put(int(35+hw-2),y,'/')
# dress folds / shoulder highlights
for y in range(27,rows):
    for x in (13,19,43,49):
        if 0<=x<cols and g[y][x] not in ' ': put(x,y,'-')
for x in range(12,51):
    if g[29][x] in ':=': put(x,29,'%')
# face ellipse
for y in range(4,21):
    for x in range(cols):
        dx=(x-35)/12.0; dy=(y-11.5)/8.0
        taper=1.0 if y<=14 else 1.0-(y-14)*0.06
        if (dx/taper)**2+dy*dy<=1:
            lx=(-dx*0.55-dy*0.35+1)/2
            g[y][x]='@' if lx>0.74 else ('#' if lx>0.5 else ('%' if lx>0.28 else '+'))
# eyes: .-o-. with dark lids
for cx,cy in ((29,11),(41,11)):
    for dx in range(-2,3): put(cx+dx,cy-1,'-',force=False)
    for ch,dx in zip('.-o-.', range(-2,3)): put(cx+dx,cy,ch)
    for dx in range(-2,3): put(cx+dx,cy+1,'.',force=False)
    put(cx-3,cy,'.'); put(cx+3,cy,'.')
# nose
for y in range(12,15): put(31,y,':')
put(31,15,'|')
put(30,16,','); put(32,16,',')
# mouth: subtle smile
for ch,x in zip("'-._.-'", range(31,38)): put(x,17,ch)
put(30,16,'.'); put(38,16,'.')
# jaw
line(24,10,26,17,'\\'); line(26,17,30,20,'\\')
line(46,10,44,17,'/'); line(44,17,40,20,'/')
for x in range(30,41): put(x,20,'-')
# hands finger lines
for k in range(4):
    line(14+k*4,47,14+k*4,56,':')
for k in range(4):
    line(36+k*4,48,36+k*4,57,':')
line(12,52,29,52,':'); line(35,53,51,53,':')
# ---- material regions used for foreground colouring --------------------
SKIN=((246,220,178),(232,196,150),(184,138,96))
HAIR=((122,82,50),(90,60,38),(56,36,24))
DRESS=((86,70,50),(54,44,34),(28,22,16))

def region_of(x,y):
    if 3<=y<=20 and ((x-35)/13.0)**2+((y-11.5)/8.5)**2<=1.05: return 'skin'
    if 19<=y<=28 and abs(x-35)<=8: return 'skin'
    if 42<=y<=55 and (((x-21)/12.0)**2+((y-49)/6.0)**2<=1.1 or ((x-49)/13.0)**2+((y-50)/5.8)**2<=1.1): return 'skin'
    if 1<=y<=26 and abs(x-35)<=22: return 'hair'
    return 'dress'

def color_for(ch,x,y):
    pal={'skin':SKIN,'hair':HAIR,'dress':DRESS}[region_of(x,y)]
    if ch in '@8#': return pal[0]
    if ch in '%M=': return pal[1]
    return pal[2]

# ---- emit the ASCII art and a per-cell colour index ---------------------
# palette indices: 1 skin hi, 2 skin, 3 skin shadow, 4 hair hi,
# 5 hair, 6 hair dark, 7 dress hi, 8 dress, 9 dress dark, A ink, B catch
def idx_for(ch, x, y):
    reg = region_of(x, y)
    if reg == 'skin':
        if ch in '@8':
            return 1
        if ch in '#M':
            return 2
        if ch in '%+':
            return 3
        if ch == 'o':
            return 0xB
        return 0xA
    if reg == 'hair':
        if ch in '%#':
            return 4
        if ch in ';:':
            return 5
        if ch in '. ,':
            return 6
        return 5
    if ch in '=-%':
        return 7
    if ch in ':.|/\\':
        return 8
    return 9

art_path = os.path.join(ROOT, 'src/data/monalisa_ascii.txt')
col_path = os.path.join(ROOT, 'src/data/monalisa_color.txt')
with open(art_path, 'w', encoding='utf-8') as fh:
    for row in g:
        fh.write(''.join(row).rstrip() + '\n')
with open(col_path, 'w', encoding='utf-8') as fh:
    for y in range(rows):
        fh.write(''.join('0123456789AB'[idx_for(g[y][x], x, y)] if g[y][x] != ' ' else '0'
                         for x in range(cols)) + '\n')
print('wrote', art_path)
print('wrote', col_path)

if len(sys.argv) > 1 and sys.argv[1] == '--preview':
    out = sys.argv[2] if len(sys.argv) > 2 else 'docs/proposals/monalisa-v2/bake-preview.png'
    img=Image.new('RGB',(cols*cw,rows*chh),(0,0,0)); dr=ImageDraw.Draw(img)
    for y in range(rows):
        for x in range(cols):
            ch=g[y][x]
            if ch!=' ':
                dr.text((x*cw,y*chh-1),ch,font=f,fill=color_for(ch,x,y))
    img.save(out); print('preview', out)
