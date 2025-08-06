#import
widgets as widgets
colors as colors

#defs
$demo_border        = 2px
$demo_br_color      = #000000
$demo_bg_color      = #009900
$nav_button_padding = 2px
$scroll_handle_color = #888888
$scroll_gutter_color = #BBBBBB

#scenes
"main_tab"
    BackgroundColor(#AAAAAA)
    FlexNode{
        flex_grow:          1
        flex_direction:     Column
        justify_self_cross: Stretch
        justify_main:       FlexStart
        row_gap:            4px
    }
    Splat<Padding>(8px)
    "header"
        FlexNode{flex_direction:Row}
        "navigation"
            FlexNode{column_gap:4px}
            Splat<Padding>(4px)
            BackgroundColor($colors::white)
            "back_button"
                NavigationButton::Back
                Splat<Padding>($nav_button_padding)
                +widgets::button{
                    "text"
                        TextLine{text:"[B]"}
                }
            "up_button"
                NavigationButton::Up
                Splat<Padding>($nav_button_padding)
                +widgets::button{
                    "text"
                        TextLine{text:"[U]"}
                }
            "next_button"
                NavigationButton::Next
                Splat<Padding>($nav_button_padding)
                +widgets::button{
                    "text"
                        TextLine{text:"[N]"}
                }
            "reload_button"
                NavigationButton::Reload
                Splat<Padding>($nav_button_padding)
                +widgets::button{
                    "text"
                        TextLine{text:"[R]"}
                }
            "location"
                ControlRoot
                Margin{left:8px right:8px top:auto bottom:auto}
                Picking::Sink
                "before"
                    ControlMember
                    BackgroundColor($colors::white)
                    TextLineColor(#000000)
                    TextLine{}
                "selected"
                    ControlMember
                    BackgroundColor($colors::text_selected)
                    TextLineColor(#000000)
                    TextLine{}
                "after"
                    ControlMember
                    BackgroundColor($colors::white)
                    TextLineColor(#000000)
                    TextLine{}
    "content"
        FlexNode{
            flex_grow:    1
            row_gap:      2px
            justify_main: FlexStart
            justify_self_cross: Stretch
        }
        BackgroundColor(#025588)
        // TODO: a more tabular view
        "overview"
            FlexNode{flex_direction:Column}
            Splat<Padding>(4px)
            "items"
                FlexNode{flex_direction:Column row_gap:4px}
        "preview"
            FlexNode{
                height:             100%
                justify_main:       FlexStart
                justify_self_cross: Stretch
                flex_grow: 6
            }
            // Splat<Margin>(8px)
            //-------------------------------------------------------------------------------------------------------------------
            // Bi-directional scrollview with scrollbars separate from content.
            //-------------------------------------------------------------------------------------------------------------------
            "scroll"
                ScrollBase
                FlexNode{
                    width:100%
                    height:100%
                    flex_direction:Column
                }
                Splat<Border>($demo_border)
                BorderColor($demo_br_color)
                BackgroundColor($demo_bg_color)
                "view_shim"
                    FlexNode{
                        width:          100%
                        flex_grow:      1
                        flex_direction: Row
                    }

                    "view"
                        ScrollView
                        FlexNode{height:100% flex_grow:1 clipping:ScrollXY}

                        // TODO: remove this extra node in bevy 0.15.1
                        "shim"
                            ScrollShim
                            AbsoluteNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
                            Splat<Padding>(4px)

                    "vertical"
                        ScrollBar{axis:Y}
                        FlexNode{height:100% width:14px}
                        BackgroundColor($scroll_handle_color)

                        "handle"
                            ScrollHandle
                            AbsoluteNode{width:100%}
                            BackgroundColor($scroll_gutter_color)

                "horizontal"
                    ScrollBar{axis:X}
                    FlexNode{width:100% height:14px}
                    BackgroundColor($scroll_handle_color)

                    "handle"
                        ScrollHandle
                        AbsoluteNode{height:100%}
                        BackgroundColor($scroll_gutter_color)

