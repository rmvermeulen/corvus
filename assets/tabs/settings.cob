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


