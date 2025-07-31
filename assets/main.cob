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
"main_tab"
    BackgroundColor(#AAAAAA)
    FlexNode{
        flex_grow:1
        flex_direction:Column
        justify_self_cross:Stretch
        justify_main:FlexStart
        row_gap:4px
    }
    Splat<Padding>(8px)
    "header"
        BackgroundColor(#AAAAAA)
        FlexNode{flex_direction:Row}
        "location"
            FlexNode{column_gap:4px}
            Splat<Padding>(4px)
            BackgroundColor(#FFFFFF)
            "back_button"
                HeaderButton::Back
                +widgets::button{
                    "text"
                        TextLine{text:"[B]"}
                }
            "reload_button"
                HeaderButton::Reload
                +widgets::button{
                    "text"
                        TextLine{text:"[R]"}
                }
            "text"
                Margin{left:8px right:8px top:auto bottom:auto}
                BackgroundColor($colors::text_selected)
                TextLineColor(#000000)
                TextLine{text:"cwd"}
    "content"
        FlexNode{flex_grow:1 column_gap:2px}
        "overview"
            FlexNode{flex_grow:1 flex_direction:Column}
            "items"
                FlexNode{flex_direction:Column row_gap:4px}
                // TODO: navigate directories
        "preview"
            // TODO: vertical scroll
            // TODO: horizontal scroll
            BackgroundColor(#558802)
            Splat<Padding>(8px)
            FlexNode{
                // min_width:      100%
                height:         100%
                // flex_grow:      2
                flex_direction: Column
                clipping:       ClipXY
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


