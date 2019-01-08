import Map hiding (units)

centre = Point 30 30
roomcentre = Point 20 30

corners =
  [ Point 18.5 37
  , Point 13 37
  , Point 13 23
  , Point 27 23
  , Point 27 37
  , Point 21.5 37
  ]


walls = Object "Walls"
  [line 0.5 p1 p2 | (p1, p2) <- zip corners (tail corners)]
box = Object "Box" [square 1 roomcentre]

units = [Point 20 28, Point 20 38]

half = Level units $ Map [walls, box]

lv = symmHalf centre half

main = putStr $ display lv "\n"
