<?php

// Function
{
    $fc = $this->_newobj();
    $out = '<<';
    $out .= ' /FunctionType 3';
    $out .= ' /Domain [0 1]';
    $functions = '';
    $bounds = '';
    $encode = '';
    $i = 1;
    $num_cols = count($grad['colors']);
    $lastcols = $num_cols - 1;

    for ($i = 1; $i < $num_cols; ++$i) {
        // predicted object id
        $functions .= ($fc + $i).' 0 R ';

        if ($i < $lastcols) {
            $bounds .= sprintf('%F ', $grad['colors'][$i]['offset']);
        }
        $encode .= '0 1 ';
    }

    $out .= ' /Functions ['.trim($functions).']';
    $out .= ' /Bounds ['.trim($bounds).']';
    $out .= ' /Encode ['.trim($encode).']';
    $out .= ' >>';
    $out .= "\n".'endobj';
    $this->_out($out);
}

// Functions
for ($i = 1; $i < $num_cols; ++$i) {
    $this->_newobj();
    $out = '<<';
    $out .= ' /FunctionType 2';
    $out .= ' /Domain [0 1]';
    $out .= ' /C0 ['.$grad['colors'][($i - 1)]['color'].']';
    $out .= ' /C1 ['.$grad['colors'][$i]['color'].']';
    $out .= ' /N '.$grad['colors'][$i]['exponent'];
    $out .= ' >>';
    $out .= "\n".'endobj';
    $this->_out($out);
}

// Shading Dictionary
{
    // set shading object
    $this->_newobj();
    $out = '<< /ShadingType '.$grad['type'];
    if (isset($grad['colspace'])) {
        $out .= ' /ColorSpace /'.$grad['colspace'];
    } else {
        $out .= ' /ColorSpace /DeviceRGB';
    }
    if (isset($grad['background']) AND !empty($grad['background'])) {
        $out .= ' /Background ['.$grad['background'].']';
    }
    if (isset($grad['antialias']) AND ($grad['antialias'] === true)) {
        $out .= ' /AntiAlias true';
    }

    $out .= ' '.sprintf('/Coords [%F %F %F %F]', $grad['coords'][0], $grad['coords'][1], $grad['coords'][2], $grad['coords'][3]);
    $out .= ' /Domain [0 1]';
    $out .= ' /Function '.$fc.' 0 R';
    $out .= ' /Extend [true true]';
    $out .= ' >>';

    $out .= "\n".'endobj';
    $this->_out($out);

    $this->gradients[$id]['id'] = $this->n;
}

// Pattern
{
    // set pattern object
    $this->_newobj();
    $out = '<< /Type /Pattern /PatternType 2';
    $out .= ' /Shading '.$this->gradients[$id]['id'].' 0 R';
    $out .= ' >>';
    $out .= "\n".'endobj';
}


$this->_out($out);
$this->gradients[$id]['pattern'] = $this->n;
