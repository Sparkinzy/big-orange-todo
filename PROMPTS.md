# 素材生图提示词（GPT-image 专用）

## 怎么用

1. **每一条都把 `assets-package/cat-body.png` 作为参考图上传**，并在提示词开头保留那句 `Using the attached image as the exact character model`。
2. 输出尺寸按下面每条的 **比例** 选，GPT-image 支持 1024×1024 / 1024×1536 / 1536×1024。
3. 生成后把图丢回项目，用 `python tools/redraw_3d_post.py` 统一去白底、裁切、缩到目标尺寸（脚本里改文件名映射即可）。
4. `peek-cat-flip.png` **不用生成**，由 peek-cat 水平镜像得到。

## 三条铁律（跑偏都出在这里）

- **比例**：Q 版大头公仔。头占整体高度 62%、比身体还宽。画成真猫骨架就是错。
- **质感**：短绒 / 植绒。细腻绒面、大面积柔光、轮廓带轻微绒毛感。**禁光滑软胶、禁塑料反光、禁陶瓷、禁粗陶土、禁长毛**。注意：只写 matte 会掉回陶土，必须同时给 `short plush pile` + `velvety` + `soft diffused light` + `subtle fuzz edge`。
- **颜色**：高饱和糖果橘。发灰、发土、偏粉都是错。

---

## 01 · peek-cat（趴窗猫）

比例 **1024×1280（竖）** → 用 1024×1536 裁

```
Using the attached image as the exact character model — same chibi short-plush kitten, same proportions, colors and surface.

A chibi short-plush kitten peeking around a vertical edge. The image is cropped so the kitten is cut off flat by the LEFT EDGE of the canvas — beyond that edge there is nothing, no wall, no panel, no column, no door frame, no shadow, just clean white. Its body is outside the frame.

Visible only:
- the huge round head, tilted about 15 degrees, facing the camera
- two front paws with white tips and toe splits, gripping over that left edge, one paw higher near the ear, one lower near the chin
- three short orange motion strokes floating to the upper right

CHIBI PROPORTIONS: head about 62% of the figure height, perfectly round and wider than the body; eyes enormous, each about 24% of the face width, nearly touching.
FACE: black pupils, pale green iris ring, three white highlights per eye (one large upper-left, two small dots), thin white lower rim; big cream muzzle filling the lower third; small pink triangular nose; dark brown wavy smile; three short orange stripes on the forehead; two orange stripes on each cheek; pink inner triangular ears; tiny pink blush ovals.
COAT: bright saturated candy orange, high chroma — not muted, not dusty, not taupe.
SURFACE: SHORT PLUSH / FLOCKED texture — a fine velvety pile, like a flocked collectible figure or soft velvet fabric. Matte but rich: the velvety nap catches light as a broad soft diffuse glow, never a sharp specular dot. The silhouette carries a subtle soft fuzz edge. NOT glossy vinyl, NOT plastic sheen, NOT ceramic, NOT raw clay, NOT rough, NOT grainy or porous, NOT long fur.
LIGHTING: bright high-key soft studio light from above-left, clean rim highlight, no cast shadow on any surface.
No outline stroke, not a flat vector, not a photograph of a real cat. Pure white background, no text, no watermark.
```

---

## 02 · cat-body（空状态趴猫）

比例 **1024×876（横）** → 用 1024×1024 裁

```
Using the attached image as the exact character model — this is the character to reproduce, same proportions, colors and surface.

A chibi short-plush kitten lying on its belly, facing the camera, front paws stretched straight forward.

CHIBI PROPORTIONS: the head is huge and perfectly round, taking about 62% of the total figure height and wider than the body; the body is a small stubby rounded blob; legs are short fat tubes; a thick segmented sausage-like tail curls up on the right side.
FACE: two ENORMOUS glossy eyes, each about 24% of the face width, nearly touching — black pupils, pale green iris ring, three white highlights per eye (one large upper-left, two small dots), thin white lower rim; big cream muzzle filling the lower third of the face; small pink triangular nose; dark brown wavy smile with smile lines; three short orange stripes on the forehead; two orange stripes on each cheek; pink inner triangular ears; tiny pink blush ovals.
BODY: white paw tips with visible toe splits on the front paws; orange tabby stripes along the back and haunches.
COAT: bright saturated candy orange, high chroma — not muted, not dusty.
SURFACE: SHORT PLUSH / FLOCKED texture — a fine velvety pile, like a flocked collectible figure or soft velvet fabric. Matte but rich: the velvety nap catches light as a broad soft diffuse glow, never a sharp specular dot. The silhouette carries a subtle soft fuzz edge. NOT glossy vinyl, NOT plastic sheen, NOT ceramic, NOT raw clay, NOT rough, NOT grainy or porous, NOT long fur.
LIGHTING: bright high-key soft studio light from above-left, clean rim highlight, one very soft short contact shadow under the body only.
Three short orange motion strokes at the upper left.
No outline stroke, not a flat vector, not a photograph of a real cat. Pure white background, no text, no watermark.
```

---

## 03 · badge-cat-0（0 条 · 窝里睡觉）

比例 **1024×1024**

```
Using the attached image as the exact character model — same chibi short-plush kitten, same colors and surface.

The chibi kitten is ASLEEP, curled up inside a round plush cat bed.

BED: shallow round basket in soft cream fleece with a padded rim, matte fabric texture with a few soft wrinkles. The bed is soft fleece; the kitten must stay short-plush.
KITTEN: curled body tucked inside, only the huge round head, part of the striped back and the curled thick tail show above the rim. Head resting on the rim, tilted slightly, one ear a little folded.
SLEEPING FACE: eyes CLOSED as smooth relaxed curved eyelids (no lashes, no lines across), peaceful expression, tiny smile, small pink triangular nose, big cream muzzle, three short orange stripes on the forehead, two orange stripes on each cheek, pink inner ears, tiny pink blush ovals.
CHIBI PROPORTIONS: the head is enormous relative to the body, about 62% of the visible figure height.
COAT: bright saturated candy orange, high chroma.
SURFACE: SHORT PLUSH / FLOCKED texture — a fine velvety pile, like a flocked collectible figure or soft velvet fabric. Matte but rich: the velvety nap catches light as a broad soft diffuse glow, never a sharp specular dot. The silhouette carries a subtle soft fuzz edge. NOT glossy vinyl, NOT plastic sheen, NOT ceramic, NOT raw clay, NOT rough, NOT grainy or porous, NOT long fur.
LIGHTING: bright high-key soft studio light from above-left, gentle soft shadow inside the bed.
NO yarn, no yarn balls.
Kitten plus bed fill about 75% of the frame width, centered.
No outline stroke, not a flat vector, not a photograph. Pure white background, no text, no watermark.
```

---

## 04–07 · badge-cat-1 ~ 4（1 / 2 / 3 / 4 个线团）

比例 **1024×1024** · 四条只有「数量 + 摆位」那两行不同，其余完全一致，**照抄不要改**。

### badge-cat-1（1 个）

```
Using the attached image as the exact character model — same chibi short-plush kitten, same proportions, colors and surface.

The chibi kitten lying on its belly, front paws stretched forward, facing the camera.

YARN: EXACTLY ONE (1) ball of soft fuzzy orange wool yarn, chunky twisted visible strands, about the size of the kitten's head, placed on the ground directly in front of the paws with a small white gap between ball and paws.

CHIBI PROPORTIONS: head about 62% of the figure height, perfectly round, wider than the body; body a small stubby blob; short fat legs; thick segmented tail curled up on the right.
FACE: enormous glossy eyes, each about 24% of the face width — black pupils, pale green iris ring, three white highlights per eye (one large upper-left, two small dots); big cream muzzle; small pink triangular nose; dark brown wavy smile; three short orange stripes on the forehead; two orange stripes per cheek; pink inner ears; tiny pink blush ovals. White paw tips with toe splits; orange stripes on the back.
COAT: bright saturated candy orange, high chroma.
SURFACE: SHORT PLUSH / FLOCKED texture — a fine velvety pile, like a flocked collectible figure or soft velvet fabric. Matte but rich: the velvety nap catches light as a broad soft diffuse glow, never a sharp specular dot. The silhouette carries a subtle soft fuzz edge. NOT glossy vinyl, NOT plastic sheen, NOT ceramic, NOT raw clay, NOT rough, NOT grainy or porous, NOT long fur.
LIGHTING: bright high-key soft studio light from above-left, one very soft short contact shadow only.
No outline stroke, not a flat vector, not a photograph. Pure white background, no text, no watermark.
```

### badge-cat-2（2 个）

把上面 YARN 那行换成：

```
YARN: EXACTLY TWO (2) balls of soft fuzzy orange wool yarn, no more no less, both the same size (about the kitten's head), one beside the left front paw and one beside the right front paw, clearly separated by white space so they can be counted instantly.
```

### badge-cat-3（3 个）

```
YARN: EXACTLY THREE (3) balls of soft fuzzy orange wool yarn, no more no less, all the same size (about the kitten's head), arranged in a neat triangle — one at the left paw, one at the right paw, one in the front centre — each clearly separated by white space so they can be counted instantly.
```

### badge-cat-4（4 个）

```
YARN: EXACTLY FOUR (4) balls of soft fuzzy orange wool yarn, no more no less, all the same size, arranged in a single row along the bottom — two left of the paws, two right of the paws, evenly spaced and clearly separated so all four can be counted instantly. The kitten's upper body stays visible above the row.
```

---

## 08 · badge-cat-5（5 条以上 · 忙昏头）

比例 **1024×1024**

```
Using the attached image as the exact character model — same chibi short-plush kitten, same proportions, colors and surface.

The chibi kitten is completely overwhelmed by too much yarn.

YARN: FIVE to SIX balls of soft fuzzy orange wool yarn of slightly different sizes, scattered in a messy pile around its feet, some behind others, overlapping. Loose twisted strands wrap around its chest and one strand drapes over the top of its head.
POSE: lying on its belly but flustered — front paws lifted apart in a helpless shrug, ears angled back, head tilted.
FACE: same enormous glossy eyes but wide open and startled, mouth open in a small round surprised "o", big cream muzzle, small pink triangular nose, three orange stripes on the forehead, two orange stripes per cheek, pink inner ears, pink blush ovals.
CHIBI PROPORTIONS: head still the dominant feature, about 60% of the figure height.
COAT: bright saturated candy orange, high chroma.
SURFACE: SHORT PLUSH / FLOCKED texture — a fine velvety pile, like a flocked collectible figure or soft velvet fabric. Matte but rich: the velvety nap catches light as a broad soft diffuse glow, never a sharp specular dot. The silhouette carries a subtle soft fuzz edge. NOT glossy vinyl, NOT plastic sheen, NOT ceramic, NOT raw clay, NOT rough, NOT grainy or porous, NOT long fur.
Two or three short curved orange motion strokes above the head to show dizzy panic — no sweat drops, no symbols, no letters.
LIGHTING: bright high-key soft studio light from above-left, one very soft short contact shadow only.
No outline stroke, not a flat vector, not a photograph. Pure white background, no text, no watermark.
```

---

## 一致性检查清单（出图后逐张对）

- [ ] 头是不是又大又圆，比身体宽
- [ ] 眼睛是不是占脸约 1/4，两点几乎挨着
- [ ] 每只眼有没有 3 个白高光点 + 浅绿虹膜圈
- [ ] 前爪有没有白手套 + 趾缝
- [ ] 背有没有橘色条纹，尾巴是不是分节上翘
- [ ] 额头三道纹、脸颊各两道斑
- [ ] 是不是短绒/植绒（绒面、柔光扩散），既不是光滑软胶也不是粗陶土
- [ ] 轮廓边缘有没有轻微绒毛感（不能是硬边塑料）
- [ ] 橘色够不够饱和（对照 cat-body.png，别发灰）
- [ ] 背景纯白、无墙、无平台、无投影
- [ ] 没有描边、不扁平

任何一条不过，别补，直接重抽 —— 补出来的会更歪。
