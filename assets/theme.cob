#import
builtin.colors.tailwind as tw

#defs

$button_text_color_idle  = $tw::AMBER_500
$button_text_color_hover = $tw::BLUE_600
$button_text_color_press = $tw::RED_600

$button_bg_color = $tw::AMBER_500

$primary = $tw::AMBER_600
$secondary = $tw::ORANGE_600
$tertiary = $tw::BLUE_600


+button = \
    Animated<BackgroundColor>{
        idle: $button_text_color_idle
        hover: $button_text_color_hover
        press: $button_text_color_press
    }
    BrRadius(8px)
    BackgroundColor($tertiary)
    // Splat<Margin>(auto)
    "text"
        Splat<Margin>(auto)
        TextLineColor(#FFFFFF)
        TextLine{text:"[button]"}
\
