module Map
( Point(..)
, Poly(..)
, Object(..)
, Map(..)
, Level(..)
, MapThing(..)

, symmHoriz
, symmVert
, symmDiag
, symmQuad
, symmHalf
, symmQuarter

, rect
, square
, line
) where

data Point = Point Float Float

data Poly = Poly { points :: [Point] }

data Object = Object { desc :: String, polys :: [Poly] }

data Map = Map [Object]

data Level = Level { units :: [Point], levelMap :: Map }

class MapThing obj where
  mapPoints :: (Point -> Point) -> (obj -> obj)

  displayLines :: obj -> [String -> String]
  displayLines x = [display x]

  display :: obj -> (String -> String)
  display = dropFormat "" "\n" "" . displayLines

format :: [a] -> [a] -> [a] ->
  [[a] -> [a]] -> [a]
format l b r [] = l ++ r
format l b r (d : ds) = l ++ d rest
  where rest = foldr (\f acc -> b ++ f acc) r ds

dropFormat :: [a] -> [a] -> [a] ->
  [[a] -> [a]] -> ([a] -> [a])
dropFormat l b r ds rd = format l b (r ++ rd) ds

instance MapThing Point where
  mapPoints = id
  display (Point x y) rest = "(" ++ show x ++ ", " ++ show y ++ ")" ++ rest

instance MapThing Poly where
  mapPoints f = Poly . map f . points
  display = dropFormat "[" ", " "]," . concatMap displayLines . points

instance MapThing Object where
  mapPoints f (Object desc polys) = Object desc (map (mapPoints f) polys)
  displayLines (Object desc polys) = comment : map display polys
    where comment rest = "// " ++ desc ++ rest

indent :: (String -> String) -> (String -> String)
indent f rest = "    " ++ f rest

instance MapThing Map where
  mapPoints f (Map objs) = Map $ map (mapPoints f) objs
  displayLines (Map objs) = format [head] [id] [last] dropObjs
    where
      head rest = "map: [" ++ rest
      last rest = "]," ++ rest
      dropObjs = map ((++) . map indent . displayLines) objs

displayUnit :: Point -> [String -> String]
displayUnit p = map (++)
 [ "("
 , "    team: 0,"
 , "    pos: " ++ display p ","
 , "    weapon: Gun,"
 , "),"
 ]

displayUnits :: [Point] -> [String -> String]
displayUnits points = head : map indent units ++ [last]
  where
    head rest = "units: [" ++ rest
    units = concatMap displayUnit points
    last rest = "]," ++ rest

instance MapThing Level where
  mapPoints f (Level units levelMap) =
    Level (map f units) (mapPoints f levelMap)
  displayLines (Level units levelMap) = head : map indent content ++ [last]
    where
      head rest = "(" ++ rest
      content = displayUnits units ++ displayLines levelMap
      last rest = ")" ++ rest



concatMapPoints :: MapThing obj => Semigroup obj =>
  (Point -> Point) -> (obj -> obj)
concatMapPoints f x = x <> mapPoints f x

foldMapPoints :: MapThing obj => Semigroup obj =>
  [Point -> Point] -> (obj -> obj)
foldMapPoints = foldr (\f g' -> concatMapPoints f . g') id

instance Semigroup Poly where
  Poly ls <> Poly rs = Poly (ls <> rs)

instance Semigroup Map where
  Map ls <> Map rs = Map (ls <> rs)

instance Semigroup Level where
  Level lu lm <> Level ru rm = Level (lu <> ru) (lm <> rm)

reflectHoriz :: Float -> Point -> Point
reflectHoriz rx (Point x y) = Point (rx + rx - x) y

reflectVert :: Float -> Point -> Point
reflectVert ry (Point x y) = Point x (ry + ry - y)

reflectDiag :: Point -> Point -> Point
reflectDiag (Point rx ry) (Point x y) = Point (rx + ry - y) (ry + rx - x)

symmHoriz :: MapThing obj => Semigroup obj => Float -> obj -> obj
symmHoriz = concatMapPoints . reflectHoriz

symmVert :: MapThing obj => Semigroup obj => Float -> obj -> obj
symmVert = concatMapPoints . reflectVert

symmDiag :: MapThing obj => Semigroup obj => Point -> obj -> obj
symmDiag = concatMapPoints . reflectDiag

symmQuad :: MapThing obj => Semigroup obj => Point -> obj -> obj
symmQuad (Point rx ry) = symmHoriz rx . symmVert ry

symmOct :: MapThing obj => Semigroup obj => Point -> obj -> obj
symmOct p = symmQuad p . symmDiag p


turnHalf :: Point -> Point -> Point
turnHalf (Point cx cy) = reflectHoriz cx . reflectVert cy

turnQuarter :: Point -> Point -> Point
turnQuarter (Point cx cy) (Point x y) = Point (cx + cy - y) (cy + x - cx)

symmHalf :: MapThing obj => Semigroup obj => Point -> obj -> obj
symmHalf = concatMapPoints . turnHalf

symmQuarter :: MapThing obj => Semigroup obj => Point -> obj -> obj
symmQuarter p = symmHalf p . concatMapPoints (turnQuarter p)



rect :: Point -> Point -> Poly
rect (Point x1 y1) (Point x2 y2) = Poly
  [ Point x1 y1
  , Point x2 y1
  , Point x2 y2
  , Point x1 y2
  ]

square :: Float -> Point -> Poly
square r (Point x y) = rect (Point (x - r) (y - r)) (Point (x + r) (y + r))

line :: Float -> Point -> Point -> Poly
line r c1 c2 = Poly [p1, p2, p3, p4, p5, p6]
  where
    Poly [p1, p2, _, p6] = square r c1
    Poly [_, p3, p4, p5] = square r c2

