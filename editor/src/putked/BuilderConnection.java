package putked;

import java.io.BufferedReader;
import java.io.DataOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.net.Socket;

public class BuilderConnection
{
	public static void onObjectChanged(DataObject object)
	{


	}

	public static void connectionThread()
	{
		do
		{
			Socket sock = null;
			try
			{
				sock = new Socket("localhost", 5555);
				DataOutputStream output = new DataOutputStream(sock.getOutputStream());
				// BufferedReader input = new BufferedReader(new InputStreamReader(sock.getInputStream()));
				output.writeBytes("<ready>\n");
				while (true)
				{
					try
					{
						Thread.sleep(250);
					}
					catch (InterruptedException ie)
					{

					}
				}
			}
			catch (IOException e)
			{

			}
			finally
			{
				if (sock != null)
				{
					try
					{
						sock.close();
					}
					catch (IOException e)
					{

					}
				}
			}
		}
		while (true);
	}
}

