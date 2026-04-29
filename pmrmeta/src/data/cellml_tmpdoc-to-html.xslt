<?xml version='1.0'?>

<xsl:stylesheet
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:tmp-doc="http://cellml.org/tmp-documentation"
    xmlns:cellml="http://www.cellml.org/cellml/1.0#"
    xmlns:cellml11="http://www.cellml.org/cellml/1.1#"
    xmlns:mathml="http://www.w3.org/1998/Math/MathML"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:cmeta="http://www.cellml.org/metadata/1.0#"
    xmlns:bqs="http://www.cellml.org/bqs/1.0#"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:dcterms="http://purl.org/dc/terms/"
    xmlns:vCard="http://www.w3.org/2001/vcard-rdf/3.0#"
    exclude-result-prefixes="tmp-doc cellml cellml11 rdf cmeta bqs dc dcterms vCard"
    version='1.0'>

  <xsl:param name="IMAGE_DIR" select="images"/>

  <xsl:output method="xml"
              encoding="ISO-8859-1"
              doctype-public="-//W3C//DTD XHTML 1.1 plus MathML 2.0//EN"
              doctype-system="http://www.w3.org/TR/MathML2/dtd/xhtml-math11-f.dtd"
              indent="no"/>

  <xsl:template match="cellml:model|cellml11:model">
    <!--
    <xsl:variable name="cmeta_id">
      <xsl:call-template name="get_cmeta_id">
        <xsl:with-param name="element" select="."/>
      </xsl:call-template>
    </xsl:variable>
    -->

    <!-- and documentation hidden away in the model -->
    <xsl:choose>
      <xsl:when test="//tmp-doc:documentation">
        <div id="tmp-documentation">
        <xsl:apply-templates select="//tmp-doc:documentation"/>
        </div>
      </xsl:when>
      <xsl:otherwise>
        <h4>
          No documentation for this model.
        </h4>
      </xsl:otherwise>
    </xsl:choose>
  </xsl:template>

  <xsl:template match="tmp-doc:documentation">
    <xsl:if test="tmp-doc:article">
      <xsl:call-template name="do-article">
        <xsl:with-param name="article-node" select="tmp-doc:article"/>
      </xsl:call-template>
    </xsl:if>
  </xsl:template><!--match="tmp-doc:documentation"-->

  <xsl:template match="*"/>
  <xsl:template match="comment()"/>

  <xsl:template name="do-head">
    <xsl:param name="name"/>
      <head>
        <title>
          <xsl:text>cellml.org model respository: </xsl:text>
          <xsl:value-of select="$name"/>
        </title>
      </head>
  </xsl:template>

  <!-- A very basic implementation to get what we want from a docbook XML
       article hacked into the CellML model -->
  <xsl:template name="do-article">
    <xsl:param name="article-node"/>
    <xsl:if test="$article-node/tmp-doc:articleinfo">
      <!-- ignore for now -->
    </xsl:if>
    <xsl:apply-templates select="$article-node/tmp-doc:section|$article-node/tmp-doc:sect1"/>
  </xsl:template>

  <xsl:template match="tmp-doc:section">
    <xsl:if test="tmp-doc:title">
      <h4>
        <xsl:value-of select="normalize-space(tmp-doc:title)"/>
      </h4>
    </xsl:if>
    <xsl:apply-templates/>
  </xsl:template>

  <xsl:template match="tmp-doc:sect1">
    <xsl:if test="tmp-doc:title">
      <h4>
        <xsl:value-of select="normalize-space(tmp-doc:title)"/>
      </h4>
    </xsl:if>
    <xsl:apply-templates/>
  </xsl:template>

  <xsl:template match="tmp-doc:sect2|tmp-doc:sect3|tmp-doc:sect4">
    <xsl:if test="tmp-doc:title">
      <p><b>
        <xsl:value-of select="normalize-space(tmp-doc:title)"/>
      </b></p>
    </xsl:if>
    <xsl:apply-templates/>
  </xsl:template>

  <xsl:template match="tmp-doc:para">
    <p class="tmp-doc-para">
      <xsl:apply-templates/>
    </p>
  </xsl:template>

  <xsl:template match="tmp-doc:itemizedlist">
    <ul class="tmp-doc-itemizedlist">
      <xsl:apply-templates/>
    </ul>
  </xsl:template>

  <xsl:template match="tmp-doc:listitem">
    <li>
      <xsl:apply-templates/>
    </li>
  </xsl:template>

  <xsl:template match="tmp-doc:markup">
    <em class="tmp-doc-markup">
      <xsl:apply-templates/>
    </em>
  </xsl:template>

  <xsl:template match="tmp-doc:emphasis">
    <em class="tmp-doc-emphasis">
      <xsl:apply-templates/>
    </em>
  </xsl:template>

  <xsl:template match="tmp-doc:superscript">
    <sup>
      <xsl:apply-templates/>
    </sup>
  </xsl:template>

  <xsl:template match="tmp-doc:subscript">
    <sub>
      <xsl:apply-templates/>
    </sub>
  </xsl:template>

  <xsl:template match="tmp-doc:xref[@linkend='fig_reaction_diagram']">
    the figure
  </xsl:template>

  <xsl:template match="tmp-doc:xref[@linkend='fig_cell_diagram']">
    the figure
  </xsl:template>

  <xsl:template match="tmp-doc:ulink">
    <a>
      <xsl:attribute name="href">
        <xsl:value-of select="@url"/>
      </xsl:attribute>
      <xsl:apply-templates/>
    </a>
  </xsl:template>

  <xsl:template match="tmp-doc:informalfigure">
    <xsl:if test="tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:imagedata/@fileref">
      <table class="tmp-doc-informalfigure">
        <tr class="tmp-doc-informalfigure">
          <td class="tmp-doc-informalfigure">
            <xsl:choose>
              <xsl:when test="tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:imagedata/@svg">

            <embed width="100%" height="660px" type="image/svg+xml" >
                  <xsl:attribute name="src">
                    <xsl:value-of select="tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:imagedata/@svg"/>
                  </xsl:attribute>
            </embed>


<!--
                <a>
                  <xsl:attribute name="href">
                    <xsl:value-of select="tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:imagedata/@svg"/>
                  </xsl:attribute>

                  <img class="tmp-doc-informalfigure">
                    <xsl:attribute name="alt">
                      <xsl:value-of select="normalize-space(tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:title)"/>
                    </xsl:attribute>
                    <xsl:attribute name="src">
                      <xsl:value-of select="tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:imagedata/@fileref"/>
                    </xsl:attribute>
                  </img>
                </a>
-->
              </xsl:when>
              <xsl:otherwise>
                <img class="tmp-doc-informalfigure">
                  <xsl:attribute name="alt">
                    <xsl:value-of select="normalize-space(tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:title)"/>
                  </xsl:attribute>
                  <xsl:attribute name="src">
                    <xsl:value-of select="tmp-doc:mediaobject/tmp-doc:imageobject/tmp-doc:imagedata/@fileref"/>
                  </xsl:attribute>
                </img>
              </xsl:otherwise>
            </xsl:choose>
          </td>
        </tr>
        <tr class="tmp-doc-informalfigure">
          <td class="tmp-doc-informalfigure-caption">
            <xsl:apply-templates select="tmp-doc:caption"/>
          </td>
        </tr>
      </table>
    </xsl:if>
  </xsl:template>

  <xsl:template match="tmp-doc:caption">
    <xsl:apply-templates/>
  </xsl:template>

  <!-- Utility templates -->

  <xsl:template name="get_cmeta_id">
    <xsl:param name="element"/>
    <xsl:if test="$element/@cmeta:id">
      <!-- need to add a # to signify that the component is in the
           current document -->
      <xsl:value-of select="concat('#',$element/@cmeta:id)"/>
    </xsl:if>
  </xsl:template>

</xsl:stylesheet>
