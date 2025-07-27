#import
colors as colors

#defs

+bg_anim = \
    Animated<BackgroundColor>{
        idle:  $colors::bg_idle
        hover: $colors::bg_hover
        press: $colors::bg_press
    }
\

+button = \
    +bg_anim{}
    BrRadius(8px)
    Marker::Button
    // BackgroundColor($colors::tertiary)
    "text"
        Splat<Margin>(auto)
        TextLineColor($colors::text_idle)
        TextLine{text:"[button]"}
\

+selected_bg_anim = \
    Multi<Animated<BackgroundColor>>[
        {
            idle:  $colors::bg_idle
            hover: $colors::bg_hover
            press: $colors::bg_press
        }
        {
            state: [Selected]
            idle:  $colors::bg_selected_idle
            hover: $colors::bg_selected_hover
            press: $colors::bg_selected_press
        }
    ]
\


+list_option = \
    RadioButton
    Marker::Option
    +selected_bg_anim{}
    TextLine{text:"[option]"}
\


+scroll = \ 
    ScrollBase
    FlexNode{ width:90% height:90% }
    "view"
        ScrollView
        FlexNode{
            min_width:100px
            min_height:100px
            width:100%
            height:100%
            flex_grow:1
            clipping:ScrollYClipX
        }
        BackgroundColor($colors::bg_menu)
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
        BackgroundColor($colors::bg_scroll_bar)
        "handle"
            ScrollHandle
            AbsoluteNode{width:100%}
            BackgroundColor($colors::bg_scroll_handle)
\

+select_list = \
    FlexNode{flex_grow:1 flex_direction:Column}
    RadioGroup
    "value"
        Marker::Label
        TextLine{text:"[value]"}
    "options"
        +scroll{}
\

+tab_button = \
    RadioButton
    Marker::Button
    FlexNode{justify_main:Center}
    Multi<Animated<BackgroundColor>>[
        {
            idle: Hsla{ hue:221 saturation:0.5 lightness:0.05 alpha:0.5 }
            hover: Hsla{ hue:21 saturation:0.7 lightness:0.05 alpha:0.5 }
        }
        {
            state: [Selected]
            idle: Hsla{ hue:221 saturation:0.5 lightness:0.20 alpha:0.5 }
            hover: Hsla{ hue:21 saturation:0.7 lightness:0.20 alpha:0.5 }
        }
    ]
    "text"
        TextLineColor(Hsla{ hue:60 saturation:0.85 lightness:0.90 alpha:1.0 })
        TextLine{text:"[tab button]"}
\

+tab_menu = \
    RadioGroup
    GridNode{grid_auto_flow:Column column_gap:10px}
\

#scenes

"button"
    +button{}

"tab_button"
    +tab_button{}

"list_option"
    +list_option{}

