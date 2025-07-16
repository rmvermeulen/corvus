#import
theme as theme

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
