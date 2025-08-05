#import
colors as colors
builtin.colors.tailwind as tw

#defs

+bg_anim = \
    Animated<BackgroundColor>{
        idle:  $tw::GRAY_500
        hover: $tw::GRAY_300
        press: $tw::GRAY_700
    }
\

+text_anim = \
    Animated<TextLineColor>{
        idle:  #FFFFFF
        hover: #888888
        press: #FFFFFF
    }
\

+tab_bg_anim = \
    Multi<Animated<BackgroundColor>>[
        {
            state: [Selected]
            idle:  $tw::YELLOW_600
            hover: $tw::YELLOW_300
            press: $tw::YELLOW_500
        },
        {
            idle:  $tw::BLUE_600
            hover: $tw::BLUE_300
            press: $tw::BLUE_500
        }
    ]
\

+tab_text_anim = \
    Multi<Animated<TextLineColor>>[
        {
            idle:  #FFFFFF
            hover: #888888
        },
        {
            state: [Selected]
            idle:  #FFFFFF
            hover: #888888
        }
    ]
\

+button = \
    +bg_anim{}
    BrRadius(8px)
    Marker::Button
    // BackgroundColor($colors::tertiary)
    ControlRoot
    "text"
        ControlMember
        Splat<Margin>(auto)
        +text_anim{}
        TextLine{text:"[button]"}
\

+selected_bg_anim = \
    Multi<Animated<BackgroundColor>>[
        {
            idle:  $tw::GRAY_300
            hover: $tw::GRAY_500
            press: $tw::GRAY_700
        }
        {
            state: [Selected]
            idle:  $tw::GRAY_400
            hover: $tw::GRAY_600
            press: $tw::GRAY_800
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
    +tab_bg_anim{}
    ControlRoot
    "text"
        ControlMember
        +text_anim{}
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

"scroll_panel"
    +scroll{}


