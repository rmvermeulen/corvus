#import
widgets as widgets
colors as colors

#defs

#scenes
"root"
    FlexNode{
        width:100vw
        height:100vh
        flex_direction:Column
    }
    Splat<Padding>(4px)
    BackgroundColor($widgets::colors::primary)
    //Actual tab menu
    "tab_buttons"
        +widgets::tab_menu{
            // TODO: set based on model
            "main"
                +widgets::tab_button{
                    "text"
                        TextLine{text:"files"}
                }
            "settings"
                +widgets::tab_button{
                    "text"
                        TextLine{text:"settings"}
                }
        }

    //This is what changes based on menu selection
    "tab_content"
        BackgroundColor(Hsla{ hue:221 saturation:0.5 lightness:0.20 alpha:0.5 })
        FlexNode{
            flex_grow:1
            // justify_main:Stretch
            justify_self_cross:Stretch
        }

    "footer"
        FlexNode{flex_direction:Row row_gap:4px}
        "refresh_button"
            +widgets::button{
                -BrRadius
                "text"
                    TextLine{text:"Refresh"}
            }
        "text"
            TextLineColor(Hsla{hue:0 saturation:0.00 lightness:0.85 alpha:1.0})
            TextLine{}

