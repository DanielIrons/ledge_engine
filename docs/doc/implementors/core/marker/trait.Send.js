(function() {var implementors = {};
implementors["ledge_engine"] = [{"text":"impl Send for Vertex","synthetic":true,"types":[]},{"text":"impl Send for InstanceData","synthetic":true,"types":[]},{"text":"impl Send for DrawInfo","synthetic":true,"types":[]},{"text":"impl Send for Rect","synthetic":true,"types":[]},{"text":"impl&lt;R&gt; !Send for DescriptorBuilder&lt;R&gt;","synthetic":true,"types":[]},{"text":"impl !Send for PipelineData","synthetic":true,"types":[]},{"text":"impl Send for BlendMode","synthetic":true,"types":[]},{"text":"impl Send for Transform","synthetic":true,"types":[]},{"text":"impl !Send for GraphicsContext","synthetic":true,"types":[]},{"text":"impl&lt;S, C&gt; Send for Shader&lt;S, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for ShaderProgram","synthetic":true,"types":[]},{"text":"impl Send for PipelineObjectSet","synthetic":true,"types":[]},{"text":"impl Send for PipelineObject","synthetic":true,"types":[]},{"text":"impl Send for VertexOrder","synthetic":true,"types":[]},{"text":"impl Send for Image","synthetic":true,"types":[]},{"text":"impl Send for PerspectiveCamera","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Send for BufferAttribute&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send + Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;Vs, Vss, Fs, Fss&gt; Send for ShaderMaterial&lt;Vs, Vss, Fs, Fss&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fs: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;Fss: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;Vs: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;Vss: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for GraphicsError","synthetic":true,"types":[]},{"text":"impl Send for WindowMode","synthetic":true,"types":[]},{"text":"impl Send for WindowSetup","synthetic":true,"types":[]},{"text":"impl Send for Conf","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Send for Handle&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for HandleId","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Send for AssetStorage&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for Texture","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()