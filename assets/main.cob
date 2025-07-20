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
"menu"
    FlexNode{flex_direction:Column}
    Splat<Padding>(8px)
    Splat<Margin>(auto)
    BrRadius(8px)
    BackgroundColor($widgets::colors::primary)
    "label"
        Splat<Margin>(auto)
        TextLineColor($widgets::colors::tertiary)
        TextLine{text:"Menu has loaded!"}

    //Actual tab menu
    "tab_menu"
        +widgets::tab_menu{
            "main"
                +widgets::tab_button{
                    "text"
                        TextLine{text:"main"}
                }
            "settings"
                +widgets::tab_button{
                    "text"
                        TextLine{text:"settings"}
                }
            "exit"
                +widgets::tab_button{
                    "text"
                        TextLine{text:"exit"}
                }
        }

    //This is what changes based on menu selection
    "tab_content"
        FlexNode{ justify_self_cross:Stretch }
        BackgroundColor(Hsla{ hue:221 saturation:0.5 lightness:0.20 alpha:0.5 })

    "footer_content"
        FlexNode{justify_main:Center}
        "text"
            TextLine{ text: "I don't change" }
            TextLineColor(Hsla{hue:0 saturation:0.00 lightness:0.85 alpha:1.0})

"main_tab"
    FlexNode{width:100% justify_main:Center}
    "buttons"
        BackgroundColor($colors::button_bg)
        FlexNode{
            width:100%
            flex_direction:Column row_gap:8px}
        Splat<Margin>(auto)
        "start"
            +widgets::button{
                Splat<Margin>(auto)
                BackgroundColor($colors::button_bg2)
                "text"
                    TextLine{text:"Start"}
            }
        "settings"
            +widgets::button{
                Splat<Margin>(auto)
                "text"
                    TextLine{text:"Settings"}
            }
        "exit"
            +widgets::button{
                Splat<Margin>(auto)
                "text"
                    TextLine{text:"Exit"}
            }

"settings_tab"
    FlexNode{flex_grow:1 justify_main:Center}
    "settings"
        BackgroundColor($colors::button_bg)
        FlexNode{
            min_width:300px
            min_height:150px
            justify_main: Center
        }
        Splat<Margin>(auto)
        Splat<Border>(2px)
        BorderColor(#000000)
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

"exit_tab"
    //Not implemented
    FlexNode{justify_main:Center}
    TextLine{text:"Click me to quit"}

"tab_button"
    +widgets::tab_button{}
