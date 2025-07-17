#import
theme as theme
builtin.colors.tailwind as tw

#defs

+list_items = \
    "text1"
        TextLine{text:"lmao1"}
    "text2"
        TextLine{text:"lmao2"}
    "text3"
        TextLine{text:"lmao3"}
    "text4"
        TextLine{text:"lmao4"}
    "text5"
        TextLine{text:"lmao5"}
    "text5"
        TextLine{text:"lmao6"}
    "text7"
        TextLine{text:"lmao7"}
    "text8"
        TextLine{text:"lmao8"}
    "text9"
        TextLine{text:"lmao9"}
    "text10"
        TextLine{text:"lmao10"}
    "text11"
        TextLine{text:"lmao11"}
    "text12"
        TextLine{text:"lmao12"}
\

+scroll = \ 
    ScrollBase
    FlexNode{
        // width:100%
        flex_grow:1
        flex_direction:Row
    }
    "view"
        ScrollView
        FlexNode{
            min_height:200px
            height:100%
            flex_grow:1
            clipping:ScrollYClipX
        }
        BackgroundColor($tw::AMBER_400)
        "shim"
            ScrollShim
            AbsoluteNode{
                flex_direction:Column
                justify_main:Center
                justify_cross:Center
            }
    "vertical"
        ScrollBar{axis:Y}
        FlexNode{height:100% width:14px}
        BackgroundColor(#888888)
        "handle"
            ScrollHandle
            AbsoluteNode{width:100%}
            BackgroundColor(#BBBBBB)
\

#scenes
"menu"
    FlexNode{flex_direction:Column}
    Splat<Padding>(8px)
    Splat<Margin>(auto)
    BrRadius(8px)
    BackgroundColor($theme::primary)
    "label"
        Splat<Margin>(auto)
        TextLineColor($theme::tertiary)
        TextLine{text:"Menu has loaded!"}
    "buttons"
        BackgroundColor($tw::AMBER_500)
        FlexNode{flex_direction:Column row_gap:8px}
        Splat<Margin>(auto)
        "start"
            +theme::button{
                Splat<Margin>(auto)
                "text"
                    TextLine{text:"Start"}
            }
        "settings"
            +theme::button{
                Splat<Margin>(auto)
                "text"
                    TextLine{text:"Settings"}
            }
        "exit"
            +theme::button{
                Splat<Margin>(auto)
                "text"
                    TextLine{text:"Exit"}
            }
    "settings"
        BackgroundColor($tw::AMBER_500)
        FlexNode{min_width:300px min_height:150px}
        Splat<Margin>(auto)
        Splat<Border>(2px)
        BorderColor(#000000)
        "left"
            +scroll{
                "view"
                    "shim"
                        +list_items{}
            }
        "right"
            +scroll{
                "view"
                    "shim"
                        +list_items{}
            } 
