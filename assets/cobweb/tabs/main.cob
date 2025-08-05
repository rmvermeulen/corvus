#import
widgets as widgets
colors as colors
#defs
$nav_button_padding = 2px
#scenes
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

