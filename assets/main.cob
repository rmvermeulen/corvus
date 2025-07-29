#import
widgets as widgets
colors as colors

#defs

+list_items = \
    "text1"
        +widgets::list_option{
            TextLine{text:"lmao1"}
        }
    "text2"
        +widgets::list_option{
            TextLine{text:"lmao2"}
        }
    "text3"
        +widgets::list_option{
            TextLine{text:"lmao3"}
        }
    "text4"
        +widgets::list_option{
            TextLine{text:"lmao4"}
        }
    "text5"
        +widgets::list_option{
            TextLine{text:"lmao5"}
        }
    "text6"
        +widgets::list_option{
            TextLine{text:"lmao6"}
        }
    "text7"
        +widgets::list_option{
            TextLine{text:"lmao7"}
        }
    "text8"
        +widgets::list_option{
            TextLine{text:"lmao8"}
        }
    "text9"
        +widgets::list_option{
            TextLine{text:"lmao9"}
        }
    "text10"
        +widgets::list_option{
            TextLine{text:"lmao10"}
        }
\

#scenes
"root"
    FlexNode{
        width:100vw
        height:100vh
        flex_direction:Column
    }
    Splat<Padding>(8px)
    BackgroundColor($widgets::colors::primary)
    "label"
        Margin{ left:auto right:auto }
        TextLineColor($widgets::colors::tertiary)
        TextLine{text:""}
    "refresh"
        +widgets::button{
            "text"
                TextLine{text:"Refresh"}
        }
    //Actual tab menu
    "tab_buttons"
        +widgets::tab_menu{
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

    "footer_content"
        FlexNode{justify_main:Center}
        "text"
            TextLine{ text: "I don't change" }
            TextLineColor(Hsla{hue:0 saturation:0.00 lightness:0.85 alpha:1.0})

"main_tab"
    BackgroundColor(#AAAAAA)
    FlexNode{flex_grow:1}
    Splat<Padding>(8px)
    "overview"
        BackgroundColor(#555555)
        FlexNode{flex_grow:1 flex_direction:Column}
        "text"
            BackgroundColor(#444444)
            TextLine{text:"Overview"}
        "items"
            BackgroundColor(#666666)
            FlexNode{flex_direction:Column row_gap:8px}
    "preview"
        BackgroundColor(#558802)
        Splat<Padding>(8px)
        FlexNode{
            min_width:      100%
            height:         100%
            flex_grow:      2
            flex_direction: Column
        }

"settings_tab"
    FlexNode{flex_grow:1 justify_main:Center}
    "settings"
        // BackgroundColor($colors::button_bg)
        FlexNode{
            min_width:300px
            width:100%
            min_height:150px
            justify_main: Center
        }
        "resolution"
            FlexNode{
                justify_self_cross: Stretch
                justify_main: FlexStart
                flex_grow:1 flex_direction:Column}
            "label"
                TextLine{text:"100x100"}
            "options"
                RadioGroup
                +widgets::scroll{
                    "view"
                        "shim"
                            // NOTE: items added from code
                }
        "foo_bar"
            +widgets::select_list{
                "value"
                    TextLine{text:"Foobar"}
                "options"
                    "view"
                        "shim"
                            +list_items{}

            }


