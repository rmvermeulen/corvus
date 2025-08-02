#import
widgets as widgets
colors as colors

#defs

+checkbox = \
    FlexNode{width:25px height:25px}
    BackgroundColor($colors::white)
    Margin{left:4px right:4px top:auto bottom:auto}
    BrRadius(100%)
    "dot"
        ControlMember
        FlexNode{width:15px height:15px}
        Splat<Margin>(auto)
        Multi<Animated<BackgroundColor>>[
            { idle: $colors::white },
            { idle: $colors::black state: [Selected] },
        ]
        BrRadius(100%)
\

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
        BackgroundColor(#AAAAAA)
        FlexNode{flex_direction:Row}
        "navigation"
            FlexNode{column_gap:4px}
            Splat<Padding>(4px)
            BackgroundColor($colors::white)
            "back_button"
                NavigationButton::Back
                +widgets::button{
                    "text"
                        TextLine{text:"[B]"}
                }
            "up_button"
                NavigationButton::Up
                +widgets::button{
                    "text"
                        TextLine{text:"[U]"}
                }
            "next_button"
                NavigationButton::Next
                +widgets::button{
                    "text"
                        TextLine{text:"[N]"}
                }
            "reload_button"
                NavigationButton::Reload
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

"settings_tab"
    FlexNode{flex_grow:1 justify_main:Center}
    "settings"
        // BackgroundColor($colors::button_bg)
        FlexNode{
            min_width:300px
            width:100%
            min_height:150px
            justify_main:SpaceEvenly
        }
        "resolution"
            FlexNode{
                justify_self_cross: Stretch
                justify_main: FlexStart
                flex_direction:Column}
            "header"
                FlexNode{}
                "label"
                    TextLine{text:"Resolution: "}
                "value"
                    TextLine{text:"100x100"}
            "options"
                RadioGroup
                +widgets::scroll{
                    "view"
                        "shim"
                            // NOTE: items added from code
                }
        "layout"
            FlexNode{
                justify_self_cross: Stretch
                justify_main: FlexStart
                flex_direction:Column}
            "header"
                FlexNode{}
                "label"
                    TextLine{text:"Layout: "}
                "value"
                    TextLine{text:"auto"}
            "options"
                FlexNode{flex_direction:Column}
                RadioGroup
                "automatic"
                    RadioButton
                    ControlRoot
                    "checkbox"
                        +checkbox{}
                    "button"
                        +widgets::button{
                            "text"
                                TextLine{text:"Automatic"}
                        }
                "horizontal"
                    RadioButton
                    ControlRoot
                    "checkbox"
                        +checkbox{}
                    "button"
                        +widgets::button{
                            "text"
                                TextLine{text:"Horizontal"}
                        }
                "vertical"
                    RadioButton
                    ControlRoot
                    "checkbox"
                        +checkbox{}
                    "button"
                        +widgets::button{
                            "text"
                                TextLine{text:"Vertical"}
                        }


