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

let Schema : Type =
      { delim : Text
      , categories : List Category
      }

-- vv  your values go here  vv --

let schema : Schema =
      { delim = "-"
      , categories =
        [ { name = "Medium"
          , rtype = Restriction.Exactly
          , rvalue = 1
          , values = ["art", "photo", "ai", "other"]
          }
        , { name = "Subject"
          , rtype = Restriction.AtLeast
          , rvalue = 0
          , values = ["plants", "animals", "people"]
          }
        ]
      }

in  schema
