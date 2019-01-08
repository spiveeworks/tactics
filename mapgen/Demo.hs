import Map

centre = Point 30 30

box = Object "Box" [rect (Point 29 29) (Point 30 30)]
roomWall = Object "Room Wall" [rect corner right, rect corner bottom]
  where
    corner = Point 20 20
    right = Point 27 21
    bottom = Point 21 27

cornerWall = Object "Corner Wall" [line 0.5 (Point 2.5 2.5) (Point 15.5 15.5)]

shieldWall = Object "Shield Wall"
  [line 0.25 (Point 10.25 20.25) (Point 17.75 27.75)]

quadrant = Level [Point 5 30] (shields <> objs)
  where
    shields = symmDiag centre (Map [shieldWall])
    objs = Map [box, roomWall, cornerWall]

lv = symmQuarter centre quadrant

main = putStr $ display lv "\n"
