{-

copy this configuration file to the directory with your media files
and edit the values of `schema` below to specify the structure of your tags

-}

let Restriction = < Exactly | AtLeast | AtMost >

let Category : Type =
      { name : Text
      , rtype : Restriction
      , rvalue : Natural
      , values : List Text
      }

let Salt : Type =
      { rtype : Restriction
      , rvalue : Natural
      , values : Text
      }

let Block = < Category: Category | Salt: Salt >

let Schema : Type =
      { delim : Text
      , blocks : List Block
      }

-- vv  your values go here  vv --

let schema : Schema =
      { delim = "-"
      , blocks =
        [ Block.Salt { rtype = Restriction.Exactly
          , rvalue = 6
          , values = "ABCDEFGHIJKLMNPQRSTUVWXYZ123456789"
          }
        , Block.Category { name = "Medium"
          , rtype = Restriction.Exactly
          , rvalue = 1
          , values = ["art", "photo", "ai", "other"]
          }
        , Block.Category { name = "Subject"
          , rtype = Restriction.AtLeast
          , rvalue = 0
          , values = ["plants", "animals", "people"]
          }
        ]
      }

in  schema
